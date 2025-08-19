pub fn rewrite_sql_with_named_params(sql: &str) -> (String, Vec<String>) {
    let mut result = String::with_capacity(sql.len());
    let mut params = Vec::new();
    let mut chars = sql.chars().peekable();
    let mut index = 0;

    while let Some(c) = chars.next() {
        if c == ':' {
            // Peek to check if this is a typecast "::"
            if chars.peek() == Some(&':') {
                // It's "::", just push one ":" and continue
                chars.next();
                result.push(':');
                result.push(':');
                continue;
            }

            // Collect identifier name
            let mut name = String::new();
            while let Some(&nc) = chars.peek() {
                if name.is_empty() {
                    if nc.is_ascii_alphabetic() || nc == '_' {
                        name.push(nc);
                        chars.next();
                    } else {
                        break; // not a valid param, just output ':'
                    }
                } else {
                    if nc.is_ascii_alphanumeric() || nc == '_' {
                        name.push(nc);
                        chars.next();
                    } else {
                        break;
                    }
                }
            }

            if !name.is_empty() {
                index += 1;
                params.push(name);
                result.push('$');
                result.push_str(&index.to_string());
                continue;
            } else {
                // lone ":" not followed by identifier
                result.push(':');
                continue;
            }
        }

        // default: copy character
        result.push(c);
    }

    (result, params)
}
