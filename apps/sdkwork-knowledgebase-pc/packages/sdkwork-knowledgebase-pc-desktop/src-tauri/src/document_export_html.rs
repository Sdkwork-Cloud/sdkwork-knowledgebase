fn escape_html_text(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn build_html_document(title: &str, body_html: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>{title}</title>
    <style>
      @page {{
        size: A4;
        margin: 18mm 16mm;
      }}
      body {{
        font-family: "Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif;
        color: #27272a;
        line-height: 1.65;
        font-size: 14px;
        background: #ffffff;
        margin: 0;
        padding: 0;
      }}
      h1 {{
        font-size: 28px;
        margin: 0 0 12px;
        color: #09090b;
        border-bottom: 1.5px solid #e4e4e7;
        padding-bottom: 12px;
        font-weight: 800;
      }}
      h2 {{
        font-size: 20px;
        margin: 28px 0 12px;
        color: #18181b;
        font-weight: 700;
        border-bottom: 1px solid #f4f4f5;
        padding-bottom: 4px;
      }}
      h3 {{
        font-size: 16px;
        margin: 22px 0 10px;
        color: #27272a;
        font-weight: 600;
      }}
      p {{
        margin: 0 0 14px;
        color: #3f3f46;
      }}
      code {{
        background: #f4f4f5;
        padding: 2px 5px;
        border-radius: 4px;
        font-family: Consolas, "Courier New", monospace;
        font-size: 0.9em;
      }}
      pre {{
        background: #f8f8f9;
        padding: 14px;
        border-radius: 8px;
        overflow-x: auto;
        font-family: Consolas, "Courier New", monospace;
        font-size: 0.88em;
        border: 1px solid #e4e4e7;
        margin: 0 0 14px;
        white-space: pre-wrap;
        word-break: break-word;
      }}
      blockquote {{
        border-left: 4px solid #4f46e5;
        margin: 0 0 14px;
        padding-left: 16px;
        color: #71717a;
        font-style: italic;
      }}
      img {{
        max-width: 100%;
        height: auto;
        border-radius: 8px;
        margin: 14px 0;
      }}
      ul, ol {{
        padding-left: 22px;
        margin: 0 0 14px;
      }}
      li {{
        margin-bottom: 6px;
      }}
      table {{
        width: 100%;
        border-collapse: collapse;
        margin: 0 0 14px;
      }}
      th, td {{
        border: 1px solid #e4e4e7;
        padding: 8px 10px;
        text-align: left;
      }}
      th {{
        background: #f8f8f9;
        font-weight: 600;
      }}
    </style>
  </head>
  <body>
    <div class="content">{body_html}</div>
  </body>
</html>"#,
        title = escape_html_text(title),
        body_html = body_html,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_html_with_title_and_body() {
        let html = build_html_document("Hello", "<p>World</p>");
        assert!(html.contains("<title>Hello</title>"));
        assert!(html.contains("<p>World</p>"));
        assert!(!html.contains("<h1>Hello</h1>"));
    }
}
