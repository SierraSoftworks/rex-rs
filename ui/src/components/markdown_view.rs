use pulldown_cmark::{Event, Options, Parser, html};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MarkdownViewProps {
    pub value: AttrValue,
    #[prop_or_default]
    pub class: Classes,
}

#[function_component(MarkdownView)]
pub fn markdown_view(props: &MarkdownViewProps) -> Html {
    let rendered = render(&props.value);

    html! {
        <div class={props.class.clone()}>
            { Html::from_html_unchecked(AttrValue::from(rendered)) }
        </div>
    }
}

/// Renders CommonMark to HTML, dropping any raw HTML the source contained.
///
/// Ideas are shared between people, so somebody else's description can end up
/// rendered in your browser. The previous interface passed raw HTML straight
/// through, which made a shared collection a way to run script in a collaborator's
/// session; discarding HTML events closes that without changing how any real
/// idea renders.
fn render(source: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let events = Parser::new_ext(source, options).filter(|event| {
        !matches!(
            event,
            Event::Html(_) | Event::InlineHtml(_) | Event::DisplayMath(_) | Event::InlineMath(_)
        )
    });

    let mut out = String::with_capacity(source.len() * 2);
    html::push_html(&mut out, events);

    out
}

#[cfg(test)]
mod tests {
    use super::render;

    #[test]
    fn renders_commonmark() {
        assert_eq!(render("*hello*"), "<p><em>hello</em></p>\n");
    }

    #[test]
    fn drops_raw_html() {
        let rendered = render("<script>alert(1)</script>\n\nhello");

        assert!(
            !rendered.contains("<script"),
            "raw HTML must not survive rendering: {rendered}"
        );
        assert!(rendered.contains("hello"));
    }

    #[test]
    fn drops_inline_html() {
        let rendered = render("a <img src=x onerror=alert(1)> b");

        assert!(!rendered.contains("onerror"), "got: {rendered}");
    }
}
