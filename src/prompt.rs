const BASE_SYSTEM: &str = r#"You are a protocol implementation assistant. You help developers understand and implement IETF RFCs. Be concise, accurate, and cite section numbers. Never invent protocol details not in the specification."#;

pub fn build_explain_prompt(rfc_num: u32, spec_text: &str) -> String {
    format!(
        r#"{system}

        RFC {rfc_num} excerpt:

        {spec_text}

        Generate TWO outputs:

        ## Part 1: Explanation (Markdown)
        Generate a clear explanation covering:
        1. What problem this protocol solves
        2. Key concepts and terminology
        3. How the protocol works (high-level flow)
        4. Common use cases

        Length: 500-800 words. Use examples where helpful.

        ## Part 2: Manim Animation Script
        Generate a complete, runnable Manim Python script that visualizes the protocol.

        Requirements:
        - Use manim.Scene as base class
        - Create 30-60 second animation showing the protocol flow
        - Use Text, Arrow, Rectangle, Circle, Lines, Networks for diagrams
        -Once components are used, clean the slate reguarly to keep it fresh
        - Make sure no overlap in between components
        - Include self.wait() between transitions
        - Add narrative text annotations
        - Show data flow, state transitions, or message exchanges and all other required sorts of animations
        - Keep it clean but informative

        Format the output as:

        ---MARKDOWN---
        [markdown explanation here]
        ---MANIM---
        [complete Python manim script here]
        ---END---"#,
                system = BASE_SYSTEM,
                rfc_num = rfc_num,
                spec_text = spec_text
            )
}

pub fn build_implement_prompt(rfc_num: u32, spec_text: &str, lang: &str, scope: &str) -> String {
    let lang_notes = get_language_notes(lang);
    let scope_desc = get_scope_description(scope);
    
    format!(
        r#"{system}

RFC {rfc_num} excerpt:

{spec_text}

Generate an implementation guide for {lang} with {scope} scope:

1. **Architecture Overview**: What components are needed
2. **Core Data Structures**: Key types (structs/classes)
3. **Code Skeleton**: Functions with TODO comments for complex logic
4. **Implementation Steps**: Suggested order to build features

{scope_desc}

{lang_notes}

CRITICAL RULES:
- Mark complex logic as TODO with section references
- Example: // TODO: Implement chunked encoding per Section 7.1
- Focus on structure and interfaces, not full implementations
- Include error handling patterns

Format: Markdown with {lang} code blocks."#,
        system = BASE_SYSTEM,
        rfc_num = rfc_num,
        spec_text = spec_text,
        lang = lang,
        scope = scope,
        scope_desc = scope_desc,
        lang_notes = lang_notes,
    )
}

fn get_scope_description(scope: &str) -> &str {
    match scope {
        "minimal" => {
            "Minimal scope: Core functionality only for demo/learning.\n\
             Example for HTTP: Handle GET only, parse basic headers, return 200 OK.\n\
             Target: ~150-200 lines, implementable in 2-3 hours."
        }
        "practical" => {
            "Practical scope: Common real-world features.\n\
             Example for HTTP: Multiple methods, 20+ headers, proper error codes.\n\
             Target: ~400-500 lines, solid foundation for projects."
        }
        _ => "Standard implementation",
    }
}

fn get_language_notes(lang: &str) -> &str {
    match lang {
        "rust" => {
            "Language-specific notes for Rust:\n\
             - Use Result<T, Box<dyn Error>> for error handling\n\
             - Use tokio for async I/O (async fn, .await)\n\
             - Prefer std::net::TcpStream or tokio::net::TcpStream\n\
             - Use Vec<u8> for byte buffers"
        }
        "python" => {
            "Language-specific notes for Python:\n\
             - Use async/await with asyncio\n\
             - Use dataclasses or NamedTuple for structures\n\
             - Use bytes/bytearray for buffers\n\
             - Raise custom exceptions for errors"
        }
        "go" => {
            "Language-specific notes for Go:\n\
             - Use net.Conn for TCP connections\n\
             - Return (result, error) tuples\n\
             - Use []byte for buffers\n\
             - Define custom error types"
        }
        _ => "",
    }
}