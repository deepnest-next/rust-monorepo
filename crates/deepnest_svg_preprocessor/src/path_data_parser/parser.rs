//use regex::Regex;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Command = 0,
    Number,
    EOD,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenType,
    text: String,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub key: char,
    pub data: Vec<f64>,
}

fn params_count_for(c: char) -> Option<usize> {
    match c {
        'A' | 'a' => Some(7),
        'C' | 'c' => Some(6),
        'H' | 'h' => Some(1),
        'L' | 'l' => Some(2),
        'M' | 'm' => Some(2),
        'Q' | 'q' => Some(4),
        'S' | 's' => Some(4),
        'T' | 't' => Some(2),
        'V' | 'v' => Some(1),
        'Z' | 'z' => Some(0),
        _ => None,
    }
}

fn tokenize(d: &str) -> Vec<Token> {
    // Estimate token count (typical SVG paths have approximately 1 token per 2-3 characters)
    let estimated_tokens = d.len() / 3;
    let mut tokens: Vec<Token> = Vec::with_capacity(estimated_tokens);
    
    // Avoid string allocation with replacing by handling commas directly in the parsing
    let mut i = 0;
    let chars: Vec<char> = d.chars().collect();
    
    while i < chars.len() {
        // Skip whitespace and commas
        while i < chars.len() && (chars[i].is_whitespace() || chars[i] == ',') {
            i += 1;
        }
        
        if i >= chars.len() {
            break;
        }
        
        // Check for commands
        let c = chars[i];
        if matches!(c, 'a' | 'A' | 'c' | 'C' | 'h' | 'H' | 'l' | 'L' | 'm' | 'M' | 
                     'q' | 'Q' | 's' | 'S' | 't' | 'T' | 'v' | 'V' | 'z' | 'Z') {
            tokens.push(Token {
                kind: TokenType::Command,
                text: c.to_string(),
            });
            i += 1;
            continue;
        }
        
        // Check for numbers
        if c == '-' || c == '+' || c == '.' || c.is_digit(10) {
            let start = i;
            
            // Handle sign
            if c == '-' || c == '+' {
                i += 1;
            }
            
            // Handle digits before decimal
            while i < chars.len() && chars[i].is_digit(10) {
                i += 1;
            }
            
            // Handle decimal point and digits after
            if i < chars.len() && chars[i] == '.' {
                i += 1;
                while i < chars.len() && chars[i].is_digit(10) {
                    i += 1;
                }
            }
            
            // Handle exponent notation
            if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                i += 1;
                if i < chars.len() && (chars[i] == '+' || chars[i] == '-') {
                    i += 1;
                }
                while i < chars.len() && chars[i].is_digit(10) {
                    i += 1;
                }
            }
            
            if i > start {
                let num_str = chars[start..i].iter().collect::<String>();
                if let Ok(num) = f64::from_str(&num_str) {
                    tokens.push(Token {
                        kind: TokenType::Number,
                        text: num.to_string(), // Use normalized representation
                    });
                } else {
                    // Skip invalid number
                    i = start + 1;
                }
            } else {
                // Skip single character that isn't a valid number start
                i += 1;
            }
            continue;
        }
        
        // Skip unknown character
        i += 1;
    }
    
    tokens.push(Token {
        kind: TokenType::EOD,
        text: String::new(),
    });
    
    tokens
}

pub fn parse_path(d: &str) -> Result<Vec<Segment>, String> {
    let tokens = tokenize(d);
    if tokens.is_empty() {
        return Err("Invalid path data: Cannot tokenize".to_string());
    }
    
    let mut segments: Vec<Segment> = Vec::new();
    let mut index = 0;
    // None entspricht dem "BOD" (Beginning Of Data)
    let mut mode: Option<char> = None;

    while index < tokens.len() {
        let token = &tokens[index];
        if token.kind == TokenType::EOD {
            break;
        }

        let params_count: usize;
        let current_mode: char;

        if mode.is_none() {
            // Beim ersten Token muss es ein "M" oder "m" sein.
            if token.text == "M" || token.text == "m" {
                mode = Some(token.text.chars().next().unwrap());
                index += 1;
                current_mode = mode.unwrap();
                params_count = params_count_for(current_mode)
                    .ok_or_else(|| format!("Unbekannter Befehl: {}", current_mode))?;
            } else {
                // Falls nicht, wird "M0,0" vorangestellt und neu geparst.
                let new_d = format!("M0,0{}", d);
                return parse_path(&new_d);
            }
        } else {
            // Bereits im Pfad: Wenn das aktuelle Token eine Zahl ist, wird der vorherige Befehl fortgeführt.
            if token.kind == TokenType::Number {
                current_mode = mode.unwrap();
                params_count = params_count_for(current_mode)
                    .ok_or_else(|| format!("Unbekannter Befehl: {}", current_mode))?;
            } else {
                // Andernfalls handelt es sich um einen neuen Befehl.
                mode = Some(token.text.chars().next().unwrap());
                current_mode = mode.unwrap();
                index += 1;
                params_count = params_count_for(current_mode)
                    .ok_or_else(|| format!("Unbekannter Befehl: {}", current_mode))?;
            }
        }

        // Überprüfe, ob genügend Tokens für die Parameter vorhanden sind.
        if index + params_count > tokens.len() {
            // Handle truncated paths gracefully - try to use what we have so far
            break;
        }

        let mut params = Vec::new();
        for i in index..index + params_count {
            if i >= tokens.len() {
                break; // Avoid going out of bounds
            }
            
            let num_token = &tokens[i];
            if num_token.kind == TokenType::Number {
                let value: f64 = match num_token.text.parse() {
                    Ok(val) => val,
                    Err(_) => {
                        // Try to sanitize the number - replace comma with period
                        let sanitized = num_token.text.replace(',', ".");
                        match sanitized.parse() {
                            Ok(val) => val,
                            Err(_) => return Err(format!(
                                "Parameter ist keine Zahl: {},{}",
                                current_mode, num_token.text
                            ))
                        }
                    }
                };
                params.push(value);
            } else {
                // Not a number - try to be forgiving and continue
                break;
            }
        }

        // Only add the segment if we got enough parameters
        // For Z/z commands we don't need any parameters
        if params.len() == params_count || (current_mode == 'Z' || current_mode == 'z') {
            let params_len = params.len();
            segments.push(Segment {
                key: current_mode,
                data: params,
            });
            index += params_len;
        } else {
            // If we didn't get enough parameters, we'll skip this command
            index += params.len();
        }

        // Nach einem "M"/"m" wird der Modus zu "L"/"l" geändert.
        if current_mode == 'M' {
            mode = Some('L');
        } else if current_mode == 'm' {
            mode = Some('l');
        }
    }
    
    if segments.is_empty() {
        return Err("No valid segments found in path".to_string());
    }
    
    Ok(segments)
}

#[allow(dead_code)]
pub fn serialize(segments: &[Segment]) -> String {
    let mut tokens: Vec<String> = Vec::new();

    for segment in segments {
        tokens.push(segment.key.to_string());
        match segment.key {
            'C' | 'c' => {
                if segment.data.len() >= 6 {
                    tokens.push(segment.data[0].to_string());
                    tokens.push(format!("{},", segment.data[1]));
                    tokens.push(segment.data[2].to_string());
                    tokens.push(format!("{},", segment.data[3]));
                    tokens.push(segment.data[4].to_string());
                    tokens.push(segment.data[5].to_string());
                }
            }
            'S' | 's' | 'Q' | 'q' => {
                if segment.data.len() >= 4 {
                    tokens.push(segment.data[0].to_string());
                    tokens.push(format!("{},", segment.data[1]));
                    tokens.push(segment.data[2].to_string());
                    tokens.push(segment.data[3].to_string());
                }
            }
            _ => {
                for d in &segment.data {
                    tokens.push(d.to_string());
                }
            }
        }
    }
    tokens.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let input = "M10,20 L30,40";
        let tokens = tokenize(input);
        // Prüft, ob das letzte Token EOD ist.
        assert_eq!(tokens.last().unwrap().kind, TokenType::EOD);
    }

    #[test]
    fn test_parse_and_serialize() {
        let path_data = "M10,20 L30,40 50,60";
        let segments = parse_path(path_data).expect("Parsing failed");
        let serialized = serialize(&segments);
        // Einfacher Test, der sicherstellt, dass etwas zurückgegeben wird.
        assert!(!serialized.is_empty());
    }
}
