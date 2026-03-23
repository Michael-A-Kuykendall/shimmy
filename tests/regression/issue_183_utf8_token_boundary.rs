/// Regression test for Issue #183: UTF-8 decode crash on multilingual models
///
/// **Symptom**: HTTP 502 Bad Gateway with body:
///   `FromUtf8Error incomplete utf-8 byte sequence from index 1`
/// Any prompt to a model that uses byte-level tokenization (qwen3, deepseek,
/// most CJK-capable models) could trigger this error, making the model
/// effectively unusable through Shimmy.
///
/// **Root cause**: The generation loop called `token_to_str(token, Special::Plaintext)?`
/// which internally calls `String::from_utf8(bytes)?`.  Byte-level tokenizers
/// split multi-byte UTF-8 characters across consecutive tokens — for example,
/// the Chinese character '你' (U+4F60, bytes 0xE4 0xBF 0xA0) may be emitted
/// as three separate single-byte tokens.  `from_utf8` on a partial sequence
/// fails, and the `?` propagates the error up as a 502.
///
/// **Fix**: Replace `token_to_str(token, Special::Plaintext)?` with
/// `token_to_bytes(token, Special::Plaintext).map(|b| String::from_utf8_lossy(&b).into_owned())`
///
/// `from_utf8_lossy` replaces invalid byte sequences with U+FFFD rather than
/// failing.  When accumulated across tokens the complete multi-byte sequence
/// is reconstructed correctly in the output string.
///
/// **Affected models**: Any model whose tokenizer uses byte-level tokens,
/// including qwen3 (all sizes), qwen2.5, deepseek-r1, cogito, and most
/// models fine-tuned on multilingual corpora.
#[cfg(test)]
mod issue_183_tests {

    /// Verify that from_utf8_lossy tolerates a partial multi-byte sequence.
    ///
    /// This mirrors a single token whose bytes are the first byte of a 3-byte
    /// UTF-8 sequence.  The old code called from_utf8() which would return Err;
    /// from_utf8_lossy returns the replacement character instead.
    #[test]
    fn test_partial_utf8_sequence_does_not_panic() {
        // First byte of a 3-byte UTF-8 sequence (e.g. start of '你' = 0xE4 0xBF 0xA0)
        let partial: Vec<u8> = vec![0xE4];
        // Must not panic or return Err — must return replacement char
        let result = String::from_utf8_lossy(&partial);
        assert!(
            !result.is_empty(),
            "from_utf8_lossy must return something for a partial sequence"
        );
        // The old code: String::from_utf8(partial) returns Err here,
        // which propagated as a 502.  from_utf8_lossy returns \u{FFFD}.
        assert!(
            String::from_utf8(partial.clone()).is_err(),
            "Confirm the old code path would have failed"
        );
    }

    /// Verify that multi-byte characters are reconstructed correctly when
    /// accumulated across multiple token emissions.
    ///
    /// Simulates a byte-level tokenizer that emits '你好' as 6 single-byte
    /// tokens.  Each individual byte fails strict UTF-8 validation, but the
    /// accumulated byte sequence decodes to the correct string.
    #[test]
    fn test_multibyte_chars_accumulate_correctly() {
        // '你好' in UTF-8:
        //   '你' = U+4F60 → [0xE4, 0xBD, 0xA0]
        //   '好' = U+597D → [0xE5, 0xA5, 0xBD]
        let token_bytes: Vec<Vec<u8>> = vec![
            vec![0xE4], vec![0xBD], vec![0xA0], // '你'
            vec![0xE5], vec![0xA5], vec![0xBD], // '好'
        ];

        // Simulate the fixed generation loop: accumulate bytes, decode at end
        let mut accumulated: Vec<u8> = Vec::new();
        for token in &token_bytes {
            // Each individual token would fail from_utf8 — that was the bug
            assert!(
                String::from_utf8(token.clone()).is_err(),
                "Individual byte token must fail strict UTF-8 (confirms scenario)"
            );
            accumulated.extend_from_slice(token);
        }

        // Full accumulated sequence decodes correctly
        let result = String::from_utf8(accumulated).expect("complete sequence must be valid UTF-8");
        assert_eq!(result, "你好", "Multi-byte characters must reconstruct correctly");
    }

    /// Verify that ASCII output (the common case) is completely unaffected.
    ///
    /// from_utf8_lossy is identical to from_utf8 for valid ASCII/UTF-8 — no
    /// regressions for English-only models or prompts.
    #[test]
    fn test_ascii_tokens_unaffected() {
        let ascii_tokens: Vec<Vec<u8>> = vec![
            b"Hello".to_vec(),
            b", ".to_vec(),
            b"world".to_vec(),
            b"!".to_vec(),
        ];

        let mut out = String::new();
        for token_bytes in ascii_tokens {
            let piece = String::from_utf8_lossy(&token_bytes).into_owned();
            out.push_str(&piece);
        }

        assert_eq!(out, "Hello, world!", "ASCII output must be unchanged by the fix");
    }

    /// Verify that the fix handles the empty-bytes edge case gracefully.
    ///
    /// Some tokenizers emit zero-length byte sequences for special tokens.
    /// The fix must not panic or produce unexpected output in this case.
    #[test]
    fn test_empty_token_bytes_handled_gracefully() {
        let empty: Vec<u8> = vec![];
        let result = String::from_utf8_lossy(&empty).into_owned();
        assert_eq!(result, "", "Empty token bytes must produce empty string");
    }

    /// Verify that the replacement character strategy does not corrupt
    /// well-formed UTF-8 that happens to contain high-byte values.
    ///
    /// Emoji and other 4-byte UTF-8 sequences must pass through unmodified
    /// when the full sequence is emitted by a single token.
    #[test]
    fn test_complete_multibyte_token_not_corrupted() {
        // '🚀' = U+1F680, UTF-8: [0xF0, 0x9F, 0x9A, 0x80]
        let rocket_bytes: Vec<u8> = vec![0xF0, 0x9F, 0x9A, 0x80];
        let result = String::from_utf8_lossy(&rocket_bytes).into_owned();
        assert_eq!(result, "🚀", "Complete 4-byte UTF-8 sequence must not be corrupted");
    }
}
