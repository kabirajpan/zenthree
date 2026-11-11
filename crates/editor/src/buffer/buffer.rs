use super::line_cache::{LineOffsetCache, PredictiveCache, ReusableBuffer};
use super::offset::Offset;
use super::point::Point;
use crate::rope::Rope;
use std::sync::Arc;

/// Buffer with advanced line offset caching for performance
/// Uses Arc for cheap cloning (copy-on-write)
#[derive(Clone)]
pub struct Buffer {
    rope: Arc<Rope>,
    line_cache: LineOffsetCache,
    reusable_buffer: ReusableBuffer,
    predictive_cache: PredictiveCache,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            rope: Arc::new(Rope::new()),
            line_cache: LineOffsetCache::new(1),
            reusable_buffer: ReusableBuffer::new(),
            predictive_cache: PredictiveCache::new(),
        }
    }

    pub fn from_text(text: &str) -> Self {
        let rope = Rope::from_text(text);
        let total_lines = if rope.is_empty() { 1 } else { rope.line_count() + 1 };

        Self {
            rope: Arc::new(rope),
            line_cache: LineOffsetCache::new(total_lines),
            reusable_buffer: ReusableBuffer::new(),
            predictive_cache: PredictiveCache::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.rope.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rope.is_empty()
    }

    pub fn line_count(&self) -> usize {
        if self.rope.is_empty() {
            1
        } else {
            self.rope.line_count() + 1
        }
    }

    pub fn to_string(&self) -> String {
        self.rope.to_string()
    }

    pub fn insert(&mut self, offset: Offset, text: &str) {
        let pos = offset.value();

        // Invalidate cache for edited line and all subsequent lines (since their byte offsets shift)
        let (line, _col) = self.rope.byte_to_line_col(pos);
        self.line_cache.invalidate_from(line);

        // Copy-on-write: only clone rope if there are other Arc references
        let rope = Arc::make_mut(&mut self.rope);
        rope.insert(pos, text);

        // Update line count in cache
        let new_line_count = if rope.is_empty() { 1 } else { rope.line_count() + 1 };
        self.line_cache.update_line_count(new_line_count);
    }

    pub fn delete(&mut self, start: Offset, end: Offset) {
        let start_pos = start.value();
        let end_pos = end.value();

        // Invalidate cache for deleted line and all subsequent lines (since their byte offsets shift)
        let (start_line, _) = self.rope.byte_to_line_col(start_pos);
        self.line_cache.invalidate_from(start_line);

        // Copy-on-write: only clone rope if there are other Arc references
        let rope = Arc::make_mut(&mut self.rope);
        rope.delete(start_pos, end_pos);

        // Update line count in cache
        let new_line_count = if rope.is_empty() { 1 } else { rope.line_count() + 1 };
        self.line_cache.update_line_count(new_line_count);
    }

    /// Get line by index - optimized via line offset cache and fast Rope slice
    pub fn line(&self, line_idx: usize) -> Option<String> {
        if line_idx >= self.line_count() {
            return None;
        }
        let start = self.line_cache.get_cached_offset(line_idx)
            .unwrap_or_else(|| self.rope.line_to_byte(line_idx));
        let end = self.line_cache.get_cached_offset(line_idx + 1)
            .unwrap_or_else(|| self.rope.line_to_byte(line_idx + 1));
        
        let mut s = self.rope.slice_bytes(start, end);
        if s.ends_with('\n') {
            s.pop();
            if s.ends_with('\r') {
                s.pop();
            }
        }
        Some(s)
    }

    /// 🚀 Zero-allocation check whether a buffer line matches `expected` (ignores trailing newlines)
    pub fn line_matches(&self, line_idx: usize, expected: &str) -> bool {
        if line_idx >= self.line_count() {
            return false;
        }
        let start = self.line_cache.get_cached_offset(line_idx)
            .unwrap_or_else(|| self.rope.line_to_byte(line_idx));
        let mut end = self.line_cache.get_cached_offset(line_idx + 1)
            .unwrap_or_else(|| self.rope.line_to_byte(line_idx + 1));

        if end > start && self.rope.byte_at(end - 1) == Some(b'\n') {
            end -= 1;
            if end > start && self.rope.byte_at(end - 1) == Some(b'\r') {
                end -= 1;
            }
        }

        if (end - start) != expected.len() {
            return false;
        }

        self.rope.slice_matches(start, end, expected)
    }

    /// 🚀 NEW: Batch get line offsets (FAST!)
    /// Use this when you need multiple line offsets at once
    pub fn get_line_offsets_batch(&mut self, lines: &[usize]) -> Vec<usize> {
        let offsets =
            self.line_cache
                .get_offsets_zero_alloc(lines, &self.rope, &mut self.reusable_buffer);
        offsets.to_vec()
    }

    /// 🚀 NEW: Ensure a range of lines is cached (for predictive scrolling)
    pub fn ensure_range_cached(&mut self, range: std::ops::Range<usize>) {
        self.line_cache.ensure_range_cached(&self.rope, range);
    }

    /// 🚀 NEW: Update scroll prediction for smart pre-caching
    pub fn update_scroll_prediction(
        &mut self,
        visible_range: std::ops::Range<usize>,
        scroll_delta: f32,
        frame_time: f32,
    ) {
        self.predictive_cache
            .update_scroll_prediction(visible_range, scroll_delta, frame_time);

        let total_lines = self.line_count();
        self.line_cache
            .predictive_ensure_cached(&self.rope, &self.predictive_cache, total_lines);

        // Memory optimization: Clean up cache if too large
        if self.line_cache.cache_stats().cached_lines > 10_000 {
            self.line_cache.smart_eviction(5_000);
        }
    }

    /// Point to offset conversion
    pub fn point_to_offset(&self, point: Point) -> Offset {
        let line_start = self.rope.line_to_byte(point.row);

        // Get column offset within line
        if let Some(line_text) = self.line(point.row) {
            let column_bytes: usize = line_text
                .chars()
                .take(point.column)
                .map(|c| c.len_utf8())
                .sum();

            Offset(line_start + column_bytes)
        } else {
            Offset(line_start)
        }
    }

    /// Offset to point conversion
    pub fn offset_to_point(&self, offset: Offset) -> Point {
        let (line, col) = self.rope.byte_to_line_col(offset.value());
        Point::new(line, col)
    }

    pub fn lines(&self) -> Vec<String> {
        let mut result = Vec::new();
        for i in 0..self.line_count() {
            if let Some(line) = self.line(i) {
                result.push(line);
            }
        }
        result
    }

    /// 🚀 Get byte range for a line (efficient - for syntax highlighting)
    pub fn line_byte_range(&self, line_idx: usize) -> Option<(usize, usize)> {
        self.rope.line_byte_range(line_idx)
    }

    /// 🚀 Extract text from byte range (efficient - no full rope conversion)
    pub fn slice_bytes(&self, start: usize, end: usize) -> String {
        self.rope.slice_bytes(start, end)
    }

    /// 🚀 Get the underlying rope reference (for advanced operations)
    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    /// 🚀 Invalidate all cached line offsets
    pub fn invalidate_cache(&mut self) {
        self.line_cache.invalidate_all();
    }

    /// 🚀 Get cache statistics for performance monitoring
    pub fn cache_stats(&self) -> String {
        let stats = self.line_cache.cache_stats();
        format!(
            "Cache: {} hits, {} misses, {:.1}% hit rate, {} lines cached",
            stats.hits,
            stats.misses,
            stats.hit_rate * 100.0,
            stats.cached_lines
        )
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
