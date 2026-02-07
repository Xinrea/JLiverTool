//! TTS message queue with priority support

use std::collections::VecDeque;

use super::TtsMessage;

/// Maximum queue size
const MAX_QUEUE_SIZE: usize = 50;

/// TTS message queue with priority support
pub struct TtsQueue {
    queue: VecDeque<TtsMessage>,
}

impl TtsQueue {
    /// Create a new TTS queue
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// Add a message to the queue
    /// If queue is full, drops the lowest priority message
    pub fn push(&mut self, message: TtsMessage) {
        if self.queue.len() >= MAX_QUEUE_SIZE {
            // Find and remove the lowest priority message
            if let Some(lowest_idx) = self.find_lowest_priority_index() {
                // Only remove if new message has higher or equal priority
                if let Some(lowest) = self.queue.get(lowest_idx) {
                    if message.priority() >= lowest.priority() {
                        self.queue.remove(lowest_idx);
                    } else {
                        // New message has lower priority than all existing, drop it
                        return;
                    }
                }
            }
        }

        // Insert in priority order (higher priority first)
        let insert_pos = self
            .queue
            .iter()
            .position(|m| m.priority() < message.priority())
            .unwrap_or(self.queue.len());

        self.queue.insert(insert_pos, message);
    }

    /// Pop the next message from the queue (highest priority first)
    pub fn pop(&mut self) -> Option<TtsMessage> {
        self.queue.pop_front()
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get the number of messages in the queue
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Clear all messages from the queue
    pub fn clear(&mut self) {
        self.queue.clear();
    }

    /// Find the index of the lowest priority message
    fn find_lowest_priority_index(&self) -> Option<usize> {
        self.queue
            .iter()
            .enumerate()
            .min_by_key(|(_, m)| m.priority())
            .map(|(idx, _)| idx)
    }
}

impl Default for TtsQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tts::TtsMessageType;

    #[test]
    fn test_queue_priority_order() {
        let mut queue = TtsQueue::new();

        // Add messages in reverse priority order
        queue.push(TtsMessage::new(TtsMessageType::Danmu, "danmu".to_string()));
        queue.push(TtsMessage::new(TtsMessageType::Gift, "gift".to_string()));
        queue.push(TtsMessage::new(
            TtsMessageType::SuperChat,
            "superchat".to_string(),
        ));

        // Should pop in priority order (SuperChat > Gift > Danmu)
        assert!(matches!(
            queue.pop().unwrap().message_type,
            TtsMessageType::SuperChat
        ));
        assert!(matches!(
            queue.pop().unwrap().message_type,
            TtsMessageType::Gift
        ));
        assert!(matches!(
            queue.pop().unwrap().message_type,
            TtsMessageType::Danmu
        ));
    }

    #[test]
    fn test_queue_max_size() {
        let mut queue = TtsQueue::new();

        // Fill queue with low priority messages
        for i in 0..60 {
            queue.push(TtsMessage::new(
                TtsMessageType::Danmu,
                format!("danmu {}", i),
            ));
        }

        // Queue should not exceed max size
        assert!(queue.len() <= 50);
    }
}
