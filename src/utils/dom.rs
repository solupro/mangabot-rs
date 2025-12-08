// src/utils/dom.rs
use scraper::{ElementRef, Node, Selector};

/// 提取元素文本，排除匹配任意 selector 的子树
pub fn text_without_any(element: &ElementRef, exclude_selectors: &[Selector]) -> Vec<String> {
    let mut texts = Vec::new();
    collect_text(*element, exclude_selectors, &mut texts);
    texts
}

fn collect_text(elem: ElementRef, exclude_selectors: &[Selector], out: &mut Vec<String>) {
    for node in elem.children() {
        match node.value() {
            Node::Text(text) => {
                let t = text.trim();
                if !t.is_empty() {
                    out.push(t.to_string());
                }
            }
            Node::Element(_) => {
                if let Some(child_elem) = ElementRef::wrap(node) {
                    // Check if this child matches any exclusion selector
                    let should_exclude =
                        exclude_selectors.iter().any(|sel| sel.matches(&child_elem));

                    if !should_exclude {
                        collect_text(child_elem, exclude_selectors, out);
                    }
                }
            }
            _ => {} // Ignore comments, doctypes etc.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    #[test]
    fn test_exclude_works() {
        let html = r#"<div>前文本<a class="tag">标签</a>后文本</div>"#;
        let doc = Html::parse_fragment(html);
        let elem = doc.select(&Selector::parse("div").unwrap()).next().unwrap();
        let exclude = [Selector::parse("a.tag").unwrap()];

        let result = text_without_any(&elem, &exclude);
        // Expect "前文本 后文本", "标签" should be excluded.
        assert_eq!(result, vec!["前文本".to_string(), "后文本".to_string()]);
    }

    #[test]
    fn test_exclude_nested() {
        let html = r#"<div>Root <span class="remove">Remove <b class="keep">ButInsideRemove</b></span> Keep</div>"#;
        let doc = Html::parse_fragment(html);
        let elem = doc.select(&Selector::parse("div").unwrap()).next().unwrap();
        let exclude = [Selector::parse(".remove").unwrap()];

        let result = text_without_any(&elem, &exclude);
        // "Remove" and "ButInsideRemove" are inside .remove, so they should be gone.
        assert_eq!(result, vec!["Root".to_string(), "Keep".to_string()]);
    }
}
