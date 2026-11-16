// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Parity vectors produced by `CPython` 3.12 (zlib 1.3.2)
//! `zlib.compress(data, 9)` — the server reference.

mod counter_tests {
    use masterror::AppResult;
    use wl_size::ZlibCounter;

    fn compressed(data: &[u8]) -> AppResult<usize> {
        ZlibCounter::new().compressed_len(data)
    }

    #[test]
    fn matches_python_reference_vectors() -> AppResult<()> {
        assert_eq!(compressed(b"")?, 8);
        assert_eq!(compressed(b"a")?, 9);
        assert_eq!(compressed(b"hello world")?, 19);
        Ok(())
    }

    #[test]
    fn matches_python_reference_repeat_ab() -> AppResult<()> {
        let repeated = b"ab".repeat(1000);
        assert_eq!(compressed(&repeated)?, 24);
        Ok(())
    }

    #[test]
    fn matches_python_reference_html_like() -> AppResult<()> {
        let repeated = b"<div class='card'><p>Hello</p></div>".repeat(40);
        assert_eq!(compressed(&repeated)?, 57);
        Ok(())
    }

    #[test]
    fn matches_python_reference_utf8() -> AppResult<()> {
        let repeated = "привет мир виджет".as_bytes().repeat(50);
        assert_eq!(compressed(&repeated)?, 54);
        Ok(())
    }

    #[test]
    fn matches_python_reference_zeros() -> AppResult<()> {
        let data = vec![0_u8; 4096];
        assert_eq!(compressed(&data)?, 26);
        Ok(())
    }

    #[test]
    fn matches_python_reference_random_lcg() -> AppResult<()> {
        let data: Vec<u8> = (0..2000_u64)
            .map(|i| ((i * 1_103_515_245 + 12345) >> 16 & 0xFF) as u8)
            .collect();
        assert_eq!(compressed(&data)?, 838);
        Ok(())
    }

    #[test]
    fn counter_is_reusable_and_deterministic() -> AppResult<()> {
        let mut counter = ZlibCounter::new();
        let first = counter.compressed_len(b"hello world")?;
        let second = counter.compressed_len(b"hello world")?;
        let third = counter.compressed_len(b"")?;
        assert_eq!(first, second);
        assert_eq!(second, 19);
        assert_eq!(third, 8);
        Ok(())
    }
}

mod status_tests {
    use masterror::AppResult;
    use wl_core::Severity;
    use wl_profile::limits::Limits;
    use wl_size::{
        SizeInfo,
        status::{DEFAULT_BUDGET, DEFAULT_MARGIN, evaluate}
    };

    fn limits() -> AppResult<Limits> {
        Ok(wl_profile::profile_1()?.limits)
    }

    #[test]
    fn clean_size_has_no_findings() -> AppResult<()> {
        let info = SizeInfo {
            raw:   1000,
            zlib9: 500
        };
        let findings = evaluate(&info, &limits()?, DEFAULT_BUDGET, DEFAULT_MARGIN);
        assert_eq!(findings, Vec::new());
        Ok(())
    }

    #[test]
    fn over_budget_warns() -> AppResult<()> {
        let info = SizeInfo {
            raw:   20000,
            zlib9: 3900
        };
        let findings = evaluate(&info, &limits()?, DEFAULT_BUDGET, DEFAULT_MARGIN);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule.id, "TGW003");
        assert_eq!(findings[0].severity, Severity::Warn);
        Ok(())
    }

    #[test]
    fn over_hard_limit_errors() -> AppResult<()> {
        let info = SizeInfo {
            raw:   31204,
            zlib9: 4231
        };
        let findings = evaluate(&info, &limits()?, DEFAULT_BUDGET, DEFAULT_MARGIN);
        assert_eq!(findings[0].rule.id, "TGW002");
        assert_eq!(findings[0].severity, Severity::Error);
        assert!(findings[0].message.contains("over by 135"));
        Ok(())
    }

    #[test]
    fn over_raw_limit_errors() -> AppResult<()> {
        let info = SizeInfo {
            raw:   40000,
            zlib9: 100
        };
        let findings = evaluate(&info, &limits()?, DEFAULT_BUDGET, DEFAULT_MARGIN);
        assert!(findings.iter().any(|f| f.rule.id == "TGW001"));
        Ok(())
    }

    #[test]
    fn near_limit_informs() -> AppResult<()> {
        let info = SizeInfo {
            raw:   30000,
            zlib9: 4060
        };
        let findings = evaluate(&info, &limits()?, DEFAULT_BUDGET, DEFAULT_MARGIN);
        assert!(findings.iter().any(|f| f.rule.id == "TGW005"));
        Ok(())
    }

    #[test]
    fn findings_sorted_by_severity() -> AppResult<()> {
        let info = SizeInfo {
            raw:   40000,
            zlib9: 4120
        };
        let findings = evaluate(&info, &limits()?, DEFAULT_BUDGET, DEFAULT_MARGIN);
        assert_eq!(findings[0].severity, Severity::Error);
        assert!(
            findings
                .last()
                .is_some_and(|f| f.severity == Severity::Info)
        );
        Ok(())
    }
}

mod default_counter {
    use masterror::AppResult;
    use wl_size::ZlibCounter;

    #[test]
    fn default_counter_matches_new() -> AppResult<()> {
        let data = b"<div>hello</div>";
        let mut default = ZlibCounter::default();
        let mut fresh = ZlibCounter::new();
        assert_eq!(default.compressed_len(data)?, fresh.compressed_len(data)?);
        Ok(())
    }
}
