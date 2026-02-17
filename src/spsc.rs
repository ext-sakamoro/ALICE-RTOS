//! Lock-free SPSC (Single-Producer, Single-Consumer) ring buffer
//!
//! Zero-copy inter-task communication. No heap, no mutex, no critical sections.
//! Uses atomic read/write indices for ISR-safe operation.
//!
//! Author: Moroya Sakamoto

use core::sync::atomic::{AtomicUsize, Ordering};

/// Lock-free SPSC ring buffer
///
/// Fixed-size, no-alloc, interrupt-safe.
/// Producer and consumer can run on different cores/priorities
/// without any locking.
pub struct SpscRing<const N: usize> {
    /// Ring buffer storage
    buffer: [u32; N],
    /// Write index (owned by producer)
    write_idx: AtomicUsize,
    /// Read index (owned by consumer)
    read_idx: AtomicUsize,
}

impl<const N: usize> SpscRing<N> {
    /// Create a new empty ring buffer
    pub const fn new() -> Self {
        Self {
            buffer: [0u32; N],
            write_idx: AtomicUsize::new(0),
            read_idx: AtomicUsize::new(0),
        }
    }

    /// Push a value (producer side)
    ///
    /// Returns false if buffer is full.
    pub fn push(&mut self, value: u32) -> bool {
        let write = self.write_idx.load(Ordering::Relaxed);
        let read = self.read_idx.load(Ordering::Acquire);
        let next_write = (write + 1) % N;

        if next_write == read {
            return false; // Full
        }

        self.buffer[write] = value;
        self.write_idx.store(next_write, Ordering::Release);
        true
    }

    /// Pop a value (consumer side)
    ///
    /// Returns None if buffer is empty.
    pub fn pop(&mut self) -> Option<u32> {
        let read = self.read_idx.load(Ordering::Relaxed);
        let write = self.write_idx.load(Ordering::Acquire);

        if read == write {
            return None; // Empty
        }

        let value = self.buffer[read];
        let next_read = (read + 1) % N;
        self.read_idx.store(next_read, Ordering::Release);
        Some(value)
    }

    /// Number of items in the buffer
    pub fn len(&self) -> usize {
        let write = self.write_idx.load(Ordering::Relaxed);
        let read = self.read_idx.load(Ordering::Relaxed);
        if write >= read {
            write - read
        } else {
            N - read + write
        }
    }

    /// Is the buffer empty?
    pub fn is_empty(&self) -> bool {
        self.write_idx.load(Ordering::Relaxed) == self.read_idx.load(Ordering::Relaxed)
    }

    /// Is the buffer full?
    pub fn is_full(&self) -> bool {
        let write = self.write_idx.load(Ordering::Relaxed);
        let read = self.read_idx.load(Ordering::Relaxed);
        (write + 1) % N == read
    }

    /// Available capacity
    pub fn capacity(&self) -> usize {
        N - 1 // One slot reserved for full/empty distinction
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.read_idx.store(0, Ordering::Relaxed);
        self.write_idx.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let mut ring = SpscRing::<8>::new();
        assert!(ring.is_empty());

        ring.push(42);
        assert_eq!(ring.len(), 1);
        assert!(!ring.is_empty());

        let val = ring.pop();
        assert_eq!(val, Some(42));
        assert!(ring.is_empty());
    }

    #[test]
    fn test_full_buffer() {
        let mut ring = SpscRing::<4>::new();
        assert!(ring.push(1));
        assert!(ring.push(2));
        assert!(ring.push(3));
        assert!(!ring.push(4)); // Full (capacity = N-1 = 3)
        assert!(ring.is_full());
    }

    #[test]
    fn test_fifo_order() {
        let mut ring = SpscRing::<8>::new();
        for i in 0..5 {
            ring.push(i);
        }
        for i in 0..5 {
            assert_eq!(ring.pop(), Some(i));
        }
    }

    #[test]
    fn test_wraparound() {
        let mut ring = SpscRing::<4>::new();
        // Fill and drain twice to test wraparound
        for round in 0..3 {
            for i in 0..3 {
                assert!(ring.push(round * 10 + i));
            }
            for i in 0..3 {
                assert_eq!(ring.pop(), Some(round * 10 + i));
            }
        }
    }

    #[test]
    fn test_capacity() {
        let ring = SpscRing::<16>::new();
        assert_eq!(ring.capacity(), 15);
    }

    #[test]
    fn test_clear() {
        let mut ring = SpscRing::<8>::new();
        ring.push(1);
        ring.push(2);
        ring.clear();
        assert!(ring.is_empty());
        assert_eq!(ring.pop(), None);
    }
}
