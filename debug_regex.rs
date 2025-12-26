use regex::Regex;

fn main() {
    let heading_regex = Regex::new(
        r#"(?s)<h([1-6])(?:.*?id=["']([^"']+)["'])?.*?>(.*?)</h[1-6]>"#
    ).unwrap();

    let html = r#"
            <h1 id="intro">Introduction</h1>
            <h2 id="overview">Overview</h2>
    "#;

    println!("Testing Heading Regex:");
    for cap in heading_regex.captures_iter(html) {
        println!("Matched: {:?}", cap);
    }

    let pre_regex = Regex::new(r#"(?s)<pre[^>]*>\s*<code[^>]*class="language-([^"]+)"[^>]*>(.*?)</code>\s*</pre>"#).unwrap();
    
    let code_html = r#"<pre><code class="language-rust">fn main() {
    println!("Hello, World!");
}</code></pre>"#;

    println!("\nTesting Pre Regex:");
    if pre_regex.is_match(code_html) {
        println!("Matched!");
        for cap in pre_regex.captures_iter(code_html) {
             println!("Capture: {:?}", cap);
        }
    } else {
        println!("Not matched!");
    }
}
