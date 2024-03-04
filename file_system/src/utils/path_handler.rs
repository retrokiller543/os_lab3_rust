use logger_macro::trace_log;

#[trace_log]
pub fn absolutize_from(path: &str, cwd: &str) -> String {
    let mut tokens: Vec<&str> = vec![];
    let mut has_change = false;

    let path_components: Vec<&str> = path.split('/').collect();
    let cwd_components: Vec<&str> = cwd.split('/').filter(|&c| !c.is_empty()).collect();

    if let Some(&first_component) = path_components.first() {
        let first_is_root = first_component.is_empty(); // Root if the first component is empty
        if first_is_root {
            tokens.push(""); // Represents the root
        } else if first_component == "." {
            has_change = true;
            tokens.extend_from_slice(&cwd_components);
        } else if first_component == ".." {
            has_change = true;
            if !cwd_components.is_empty() {
                tokens.extend_from_slice(&cwd_components[..cwd_components.len() - 1]);
            }
        } else {
            has_change = true;
            tokens.extend_from_slice(&cwd_components);
            tokens.push(first_component);
        }

        for &component in &path_components[1..] {
            match component {
                "." => has_change = true,
                ".." => {
                    has_change = true;
                    if !tokens.is_empty() && tokens.last() != Some(&"") {
                        // Not to pop the root
                        tokens.pop();
                    }
                }
                _ => tokens.push(component),
            }
        }
    }

    if tokens.is_empty() || (tokens.len() == 1 && tokens[0].is_empty()) {
        "/".to_string()
    } else {
        let result_path = tokens.join("/");
        if has_change {
            result_path
        } else {
            path.to_string()
        }
    }
}

#[trace_log]
pub fn split_path(path: String) -> (String, String) {
    // To find the parent directory path
    let parts: Vec<&str> = path.split('/').collect();
    let parent_parts = &parts[..parts.len() - 1]; // Exclude the last element
    let parent = parent_parts.join("/");

    // For an empty path or root, you might want to handle it differently
    let parent = if parent.is_empty() { "/" } else { &parent };

    // To get the name of the file or directory
    let name = parts.last().unwrap_or(&""); // Safely get the last part, or default to an empty string
    (parent.to_string(), name.to_string())
}
