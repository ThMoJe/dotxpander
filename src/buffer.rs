/// A fixed-size circular buffer for tracking recent keystrokes.
/// 
/// Used by the keyboard hook to maintain a rolling window of typed characters
/// for matching against snippet triggers. Zero heap allocation after construction.
pub struct KeyBuffer {
    buffer: Vec<char>,
    capacity: usize,
    head: usize,   // Points to the next write position
    len: usize,    // Current number of valid chars
}

impl KeyBuffer {
    /// Creates a new buffer with the specified capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec!['\0'; capacity],
            capacity,
            head: 0,
            len: 0,
        }
    }

    /// Appends a character, wrapping if full (oldest char lost).
    pub fn push(&mut self, ch: char) {
        if self.capacity == 0 {
            return;
        }
        self.buffer[self.head] = ch;
        self.head = (self.head + 1) % self.capacity;
        if self.len < self.capacity {
            self.len += 1;
        }
    }

    /// Removes and returns the most recent character.
    pub fn pop(&mut self) -> Option<char> {
        if self.len == 0 {
            return None;
        }
        self.head = if self.head == 0 {
            self.capacity - 1
        } else {
            self.head - 1
        };
        self.len -= 1;
        Some(self.buffer[self.head])
    }

    /// Empties the buffer.
    pub const fn clear(&mut self) {
        self.head = 0;
        self.len = 0;
    }

    /// Returns the current number of characters in the buffer.
    #[must_use]
    #[allow(dead_code)]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the buffer is empty.
    #[must_use]
    #[allow(dead_code)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns true if the buffer content ends with the given trigger string.
    /// 
    /// Optimized for zero heap allocations and instant early-exit on mismatch.
    #[must_use]
    #[inline]
    pub fn ends_with(&self, trigger: &str) -> bool {
        if trigger.is_empty() {
            return true;
        }

        if self.len == 0 || self.capacity == 0 {
            return false;
        }

        // Check backwards from the most recently inserted character
        let mut curr = if self.head == 0 {
            self.capacity - 1
        } else {
            self.head - 1
        };

        let mut count = 0;
        for ch in trigger.chars().rev() {
            count += 1;
            if count > self.len {
                return false;
            }
            if self.buffer[curr] != ch {
                return false;
            }
            curr = if curr == 0 {
                self.capacity - 1
            } else {
                curr - 1
            };
        }

        true
    }

    /// Returns the current buffer content as a String (for debugging).
    #[must_use]
    pub fn content(&self) -> String {
        if self.len == 0 {
            return String::new();
        }

        let start = if self.len < self.capacity {
            0
        } else {
            self.head
        };

        let mut res = String::with_capacity(self.len);
        for i in 0..self.len {
            let idx = (start + i) % self.capacity;
            res.push(self.buffer[idx]);
        }
        res
    }

    /// Writes the current buffer content into an existing `String`, reusing its
    /// allocated capacity to avoid a heap allocation.
    ///
    /// This is the zero-allocation alternative to `content()` for use in
    /// `update_buffer_debug`. The target string is cleared first, then
    /// characters are appended in order. If the string already has sufficient
    /// capacity no allocation occurs.
    pub fn write_content_to(&self, out: &mut String) {
        out.clear();
        if self.len == 0 {
            return;
        }
        let start = if self.len < self.capacity { 0 } else { self.head };
        for i in 0..self.len {
            let idx = (start + i) % self.capacity;
            out.push(self.buffer[idx]);
        }
    }

    /// Returns the buffer's maximum capacity.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Resizes the circular buffer to a new capacity.
    ///
    /// Preserves the most recent characters up to `new_capacity`. If the buffer
    /// is shrinking, older characters are discarded.
    pub fn resize(&mut self, new_capacity: usize) {
        if new_capacity == self.capacity {
            return;
        }

        if new_capacity == 0 {
            self.buffer = Vec::new();
            self.capacity = 0;
            self.head = 0;
            self.len = 0;
            return;
        }

        // Collect existing valid characters in chronological order
        let mut chars = Vec::with_capacity(self.len);
        if self.len > 0 {
            let start = if self.len < self.capacity { 0 } else { self.head };
            for i in 0..self.len {
                let idx = (start + i) % self.capacity;
                chars.push(self.buffer[idx]);
            }
        }

        // Retain only the most recent characters that fit into new_capacity
        let kept = if chars.len() > new_capacity {
            let skip = chars.len() - new_capacity;
            &chars[skip..]
        } else {
            &chars[..]
        };

        let mut new_buf = vec!['\0'; new_capacity];
        for (i, &ch) in kept.iter().enumerate() {
            new_buf[i] = ch;
        }

        self.buffer = new_buf;
        self.capacity = new_capacity;
        self.head = kept.len() % new_capacity;
        self.len = kept.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_content() {
        let mut buf = KeyBuffer::new(5);
        buf.push('a');
        buf.push('b');
        buf.push('c');
        assert_eq!(buf.content(), "abc");
    }

    #[test]
    fn test_overflow() {
        let mut buf = KeyBuffer::new(3);
        buf.push('a');
        buf.push('b');
        buf.push('c');
        buf.push('d');
        assert_eq!(buf.content(), "bcd");
        assert_eq!(buf.len(), 3);
    }

    #[test]
    fn test_ends_with() {
        let mut buf = KeyBuffer::new(10);
        buf.push('h');
        buf.push('e');
        buf.push('l');
        buf.push('l');
        buf.push('o');
        assert!(buf.ends_with("llo"));
        assert!(buf.ends_with("hello"));
        assert!(!buf.ends_with("hell"));
        assert!(!buf.ends_with("hello!"));
        assert!(buf.ends_with(""));
    }

    #[test]
    fn test_pop() {
        let mut buf = KeyBuffer::new(3);
        buf.push('a');
        buf.push('b');
        assert_eq!(buf.pop(), Some('b'));
        assert_eq!(buf.content(), "a");
        buf.push('c');
        assert_eq!(buf.content(), "ac");
        assert_eq!(buf.pop(), Some('c'));
        assert_eq!(buf.pop(), Some('a'));
        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn test_clear() {
        let mut buf = KeyBuffer::new(3);
        buf.push('a');
        buf.clear();
        assert_eq!(buf.len(), 0);
        assert!(buf.is_empty());
        assert_eq!(buf.content(), "");
    }

    #[test]
    fn test_resize_expand() {
        let mut buf = KeyBuffer::new(3);
        buf.push('a');
        buf.push('b');
        buf.resize(5);
        assert_eq!(buf.capacity(), 5);
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.content(), "ab");
        buf.push('c');
        buf.push('d');
        assert_eq!(buf.content(), "abcd");
    }

    #[test]
    fn test_resize_shrink() {
        let mut buf = KeyBuffer::new(5);
        buf.push('a');
        buf.push('b');
        buf.push('c');
        buf.push('d');
        buf.push('e');
        buf.resize(3);
        assert_eq!(buf.capacity(), 3);
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.content(), "cde");
        buf.push('f');
        assert_eq!(buf.content(), "def");
    }

    #[test]
    fn test_resize_shrink_with_wraparound() {
        let mut buf = KeyBuffer::new(3);
        buf.push('a');
        buf.push('b');
        buf.push('c');
        buf.push('d'); // buffer contains "bcd", head at 1
        buf.resize(2);
        assert_eq!(buf.capacity(), 2);
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.content(), "cd");
        buf.push('e');
        assert_eq!(buf.content(), "de");
    }

    #[test]
    fn test_resize_same_capacity() {
        let mut buf = KeyBuffer::new(3);
        buf.push('a');
        buf.resize(3);
        assert_eq!(buf.capacity(), 3);
        assert_eq!(buf.content(), "a");
    }

    #[test]
    fn test_resize_empty_buffer() {
        let mut buf = KeyBuffer::new(5);
        buf.resize(10);
        assert_eq!(buf.capacity(), 10);
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.content(), "");
        buf.push('x');
        assert_eq!(buf.content(), "x");
    }

    #[test]
    fn test_resize_to_zero() {
        let mut buf = KeyBuffer::new(3);
        buf.push('a');
        buf.push('b');
        buf.resize(0);
        assert_eq!(buf.capacity(), 0);
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.content(), "");
        buf.push('c');
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_write_content_to() {
        let mut buf = KeyBuffer::new(4);
        buf.push('w');
        buf.push('x');
        buf.push('y');
        buf.push('z');
        let mut out = String::new();
        buf.write_content_to(&mut out);
        assert_eq!(out, "wxyz");

        // Shrink and write again
        buf.resize(2);
        buf.write_content_to(&mut out);
        assert_eq!(out, "yz");
    }
}
