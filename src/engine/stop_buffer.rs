#[derive(Debug)]
pub(crate) struct StopBuffer {
    stops: Vec<String>,
    pending: String,
    output: String,
    stopped: bool,
}
impl StopBuffer {
    pub(crate) fn new(stops: &[String]) -> Result<Self, crate::prepared_prompt::PromptError> {
        if stops.iter().any(String::is_empty) {
            return Err(crate::prepared_prompt::PromptError::StopPolicy);
        }
        Ok(Self {
            stops: stops.to_vec(),
            pending: String::new(),
            output: String::new(),
            stopped: false,
        })
    }
    pub(crate) fn push(&mut self, piece: &str) -> String {
        if self.stopped {
            return String::new();
        }
        self.pending.push_str(piece);
        if let Some(position) = self.stops.iter().filter_map(|s| self.pending.find(s)).min() {
            let emitted = self.pending[..position].to_string();
            self.output.push_str(&emitted);
            self.pending.clear();
            self.stopped = true;
            return emitted;
        }
        let max = self
            .stops
            .iter()
            .map(String::len)
            .max()
            .unwrap_or(0)
            .saturating_sub(1)
            .min(self.pending.len());
        let hold = (1..=max)
            .rev()
            .find(|len| {
                let start = self.pending.len() - len;
                self.pending.is_char_boundary(start)
                    && self
                        .stops
                        .iter()
                        .any(|s| s.starts_with(&self.pending[start..]))
            })
            .unwrap_or(0);
        let emitted = self.pending[..self.pending.len() - hold].to_string();
        self.pending.drain(..self.pending.len() - hold);
        self.output.push_str(&emitted);
        emitted
    }
    pub(crate) fn finish(&mut self) -> String {
        if self.stopped {
            return String::new();
        }
        let tail = std::mem::take(&mut self.pending);
        self.output.push_str(&tail);
        tail
    }
    pub(crate) fn stopped(&self) -> bool {
        self.stopped
    }
    pub(crate) fn output(&self) -> &str {
        &self.output
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn split_stop_is_withheld_and_removed() {
        let mut b = StopBuffer::new(&["STOP".into()]).unwrap();
        assert_eq!(b.push("hello ST"), "hello ");
        assert_eq!(b.push("OP ignored"), "");
        assert!(b.stopped());
        assert_eq!(b.output(), "hello ");
    }
    #[test]
    fn partial_nonmatching_stop_is_flushed() {
        let mut b = StopBuffer::new(&["stop".into()]).unwrap();
        assert_eq!(b.push("test st"), "test ");
        assert_eq!(b.finish(), "st");
        assert_eq!(b.output(), "test st");
    }
    #[test]
    fn unicode_stop_and_overlap_preserve_bytes() {
        let mut b = StopBuffer::new(&["你好".into(), "END".into()]).unwrap();
        assert_eq!(b.push("a你"), "a");
        assert_eq!(b.push("好z"), "");
        assert_eq!(b.output(), "a");
    }
}
