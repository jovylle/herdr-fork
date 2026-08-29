use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub(crate) fn display_width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

pub(crate) fn display_width_u16(text: &str) -> u16 {
    display_width(text).min(u16::MAX as usize) as u16
}

pub(crate) fn truncate_end(text: &str, max_width: usize) -> String {
    if display_width(text) <= max_width {
        return text.to_string();
    }
    if max_width == 0 {
        return String::new();
    }
    if max_width == 1 {
        return "…".to_string();
    }

    let prefix = take_prefix_width(text, max_width.saturating_sub(1));
    format!("{prefix}…")
}

pub(crate) fn middle_elide(text: &str, max_width: usize) -> String {
    if display_width(text) <= max_width {
        return text.to_string();
    }
    if max_width <= 1 {
        return "…".to_string();
    }

    let content_width = max_width.saturating_sub(1);
    let left_width = content_width / 2;
    let right_width = content_width.saturating_sub(left_width);
    let prefix = take_prefix_width(text, left_width);
    let suffix = take_suffix_width(text, right_width);
    format!("{prefix}…{suffix}")
}

fn take_prefix_width(text: &str, max_width: usize) -> String {
    let mut output = String::new();
    let mut width = 0usize;
    for ch in text.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + ch_width > max_width {
            break;
        }
        output.push(ch);
        width += ch_width;
    }
    output
}

fn take_suffix_width(text: &str, max_width: usize) -> String {
    let mut output = Vec::new();
    let mut width = 0usize;
    for ch in text.chars().rev() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + ch_width > max_width {
            break;
        }
        output.push(ch);
        width += ch_width;
    }
    output.into_iter().rev().collect()
}

pub(crate) fn softwrap(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return Vec::new();
    }
    if display_width(text) <= max_width {
        return vec![text.to_string()];
    }
    // Split by whitespace into words, then pack greedily.
    // Hard-break any word longer than max_width.
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return vec![String::new()];
    }
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut cur_width = 0usize;
    for word in words {
        let w = display_width(word);
        if w > max_width {
            // word itself too long: flush current then hard-break word
            if !current.is_empty() {
                lines.push(std::mem::take(&mut current));
                cur_width = 0;
            }
            let mut chunk = String::new();
            let mut chunk_w = 0usize;
            for ch in word.chars() {
                let cw = UnicodeWidthChar::width(ch).unwrap_or(0);
                if chunk_w + cw > max_width {
                    lines.push(std::mem::take(&mut chunk));
                    chunk_w = 0;
                }
                chunk.push(ch);
                chunk_w += cw;
            }
            if !chunk.is_empty() {
                current = chunk;
                cur_width = display_width(&current);
            }
            continue;
        }
        let sep_w = if current.is_empty() { 0 } else { 1 };
        if cur_width + sep_w + w <= max_width {
            if !current.is_empty() {
                current.push(' ');
                cur_width += 1;
            }
            current.push_str(word);
            cur_width += w;
        } else {
            lines.push(std::mem::take(&mut current));
            current = word.to_string();
            cur_width = w;
        }
    }
    if !current.is_empty() || lines.is_empty() {
        lines.push(current);
    }
    // If original text had no whitespace (single word) we already hard-broke,
    // join should equal original without inserted spaces — verified by joining
    // chunks directly.
    lines.retain(|l| !l.is_empty());
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_end_uses_display_width() {
        let text = truncate_end("提交 herdr 的反馈", 16);

        assert_eq!(text, "提交 herdr 的反…");
        assert!(display_width(&text) <= 16);
    }

    #[test]
    fn middle_elide_uses_display_width() {
        let text = middle_elide("重构用户认证模块并迁移到统一登录服务", 12);

        assert!(text.contains('…'));
        assert!(display_width(&text) <= 12);
    }

    #[test]
    fn softwrap_ascii_word_boundary() {
        let lines = softwrap("hello world from herdr", 10);
        assert_eq!(lines, vec!["hello", "world from", "herdr"]);
        for l in &lines {
            assert!(display_width(l) <= 10);
        }
    }

    #[test]
    fn softwrap_hard_break_long_token() {
        let lines = softwrap("superlongworkspacename", 8);
        assert!(lines.len() > 1);
        for l in &lines {
            assert!(display_width(l) <= 8);
        }
        assert_eq!(lines.join(""), "superlongworkspacename");
    }

    #[test]
    fn softwrap_unicode_width() {
        let lines = softwrap("提交 herdr 的反馈 12345", 8);
        for l in &lines {
            assert!(display_width(l) <= 8);
        }
    }
}
