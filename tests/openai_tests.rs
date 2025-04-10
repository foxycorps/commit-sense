use anyhow::Result;
use commit_sense::openai::*;
use commit_sense::ProjectType;
use mockito::Server;

// Helper function for extracting JSON blocks from text
fn extract_json_block(text: &str) -> Option<String> {
    // Look for code fence blocks (```json ... ```)
    if let Some(start) = text.find("```json") {
        let start_content = start + "```json".len();
        if let Some(end) = text[start_content..].find("```") {
            let block = text[start_content..start_content + end].trim();
            return Some(block.to_string());
        }
    }
    
    // Look for JSON objects directly ({ ... })
    if let Some(start) = text.find('{') {
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;
        
        for (i, c) in text[start..].char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }
            
            match c {
                '\\' => escape_next = true,
                '"' => in_string = !in_string,
                '{' if !in_string => depth += 1,
                '}' if !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(text[start..start + i + 1].to_string());
                    }
                }
                _ => {}
            }
        }
    }
    
    None
}

#[test]
fn test_openai_client_empty_commits() -> Result<()> {
    // Setup mock server
    let mut server = Server::new();
    let _mock = server.mock("POST", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "choices": [
                {
                    "message": {
                        "role": "assistant",
                        "content": "```json\n{\"bump\": \"none\", \"next_version\": \"1.0.0\", \"changelog\": \"No changes\"}\n```"
                    }
                }
            ]
        }"#)
        .create();
    
    // Initialize client with mock server
    let client = OpenAIClient::new(
        "fake_api_key".to_string(), 
        server.url(), 
        "gpt-3.5-turbo".to_string()
    );
    
    // Call with empty commits
    let result = client.get_version_and_changelog("1.0.0", &[], ProjectType::Rust);
    
    // Should return a valid result with "none" bump
    assert!(result.is_ok());
    let suggestion = result.unwrap();
    assert_eq!(suggestion.bump_type, "none");
    assert_eq!(suggestion.next_version, "1.0.0");
    assert_eq!(suggestion.changelog_markdown, "No changes");
    
    Ok(())
}

#[test]
fn test_extract_json_block_code_fence() {
    let text = r#"
Here's my analysis:

```json
{
  "bump": "minor",
  "next_version": "1.2.0",
  "changelog": "- Added new feature\n- Fixed bug"
}
```

Hope this helps!
"#;

    let json = extract_json_block(text);
    assert!(json.is_some());
    let json = json.unwrap();
    assert!(json.contains("\"bump\": \"minor\""));
    assert!(json.contains("\"next_version\": \"1.2.0\""));
}

#[test]
fn test_extract_json_block_direct() {
    let text = r#"
Based on the commits, I suggest the following version bump:

{
  "bump": "patch",
  "next_version": "1.0.1",
  "changelog": "- Fixed typo in README"
}

Let me know if you need anything else.
"#;

    let json = extract_json_block(text);
    assert!(json.is_some());
    let json = json.unwrap();
    assert!(json.contains("\"bump\": \"patch\""));
    assert!(json.contains("\"next_version\": \"1.0.1\""));
}

#[test]
fn test_openai_client_successful_response() -> Result<()> {
    // Setup mock server
    let mut server = Server::new();
    let _mock = server.mock("POST", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "choices": [
                {
                    "message": {
                        "role": "assistant",
                        "content": "```json\n{\"bump\": \"minor\", \"next_version\": \"1.1.0\", \"changelog\": \"- Added new feature X\\n- Improved performance\"}\n```"
                    }
                }
            ]
        }"#)
        .create();
    
    // Initialize client with mock server
    let client = OpenAIClient::new(
        "fake_api_key".to_string(), 
        server.url(), 
        "gpt-3.5-turbo".to_string()
    );
    
    // Call with some commits
    let commits = vec![
        "feat: Add feature X".to_string(),
        "perf: Improve performance".to_string()
    ];
    
    let result = client.get_version_and_changelog("1.0.0", &commits, ProjectType::Rust);
    
    // Debug output
    println!("Result: {:?}", result);
    if let Err(ref e) = result {
        println!("Error: {:?}", e);
    }
    
    // Should return a valid result with "minor" bump
    assert!(result.is_ok());
    let suggestion = result.unwrap();
    assert_eq!(suggestion.bump_type, "minor");
    assert_eq!(suggestion.next_version, "1.1.0");
    assert!(suggestion.changelog_markdown.contains("Added new feature X"));
    
    Ok(())
}

#[test]
fn test_openai_client_malformed_response() -> Result<()> {
    // Setup mock server with a malformed response (no JSON in content)
    let mut server = Server::new();
    let _mock = server.mock("POST", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "choices": [
                {
                    "message": {
                        "role": "assistant",
                        "content": "I'm not sure how to respond with JSON. Could you help me?"
                    }
                }
            ]
        }"#)
        .create();
    
    // Initialize client with mock server
    let client = OpenAIClient::new(
        "fake_api_key".to_string(), 
        server.url(), 
        "gpt-3.5-turbo".to_string()
    );
    
    // Call with some commits
    let commits = vec!["test: Add test".to_string()];
    
    let result = client.get_version_and_changelog("1.0.0", &commits, ProjectType::Rust);
    
    // Should return an error
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_openai_client_api_error() -> Result<()> {
    // Setup mock server with an API error
    let mut server = Server::new();
    let _mock = server.mock("POST", "/")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "error": {
                "message": "API Error for testing"
            }
        }"#)
        .create();
    
    // Initialize client with mock server
    let client = OpenAIClient::new(
        "invalid_api_key".to_string(), 
        server.url(), 
        "gpt-3.5-turbo".to_string()
    );
    
    // Call with some commits
    let commits = vec!["docs: Update docs".to_string()];
    
    let result = client.get_version_and_changelog("1.0.0", &commits, ProjectType::Rust);
    
    // Should return an error
    assert!(result.is_err());
    
    Ok(())
}
