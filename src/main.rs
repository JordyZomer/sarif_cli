use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

use colored::*;
use serde_json;

use tree_sitter::{Language, Node, Parser, Tree};

extern "C" {
    fn tree_sitter_c() -> Language;
}

struct Alert {
    file: String,
    line: usize,
    column: usize,
    message: String,
}

fn parse_alerts(file: &str, source_dir: &str) -> Vec<Alert> {
    let file = File::open(file).unwrap();
    let reader = BufReader::new(file);

    let mut alerts: Vec<Alert> = Vec::new();

    let json_file: serde_json::Value = serde_json::from_reader(reader).unwrap();

    let results = &json_file["runs"][0]["results"];

    for result in results.as_array().unwrap() {
        let message = &result["message"]["text"];

        let locations = &result["locations"];
        let locations = locations.as_array().unwrap();

        for location in locations {
            let physical_location = &location["physicalLocation"];
            let physical_location = physical_location.as_object().unwrap();
            let artifact_location = &physical_location["artifactLocation"];
            let artifact_location = artifact_location.as_object().unwrap();
            let artifact_uri = &artifact_location["uri"];
            let artifact_uri = artifact_uri.as_str().unwrap();
            let artifact_uri = artifact_uri.to_string();
            let artifact_uri = artifact_uri.replace("file://", "");
            let artifact_uri = artifact_uri.replace("%20", " ");
            let artifact_uri = format!("{}/{}", source_dir, artifact_uri);

            let line_number = &location["physicalLocation"]["region"]["startLine"];
            let line_number = line_number.as_i64().unwrap() as usize;

            let column = &location["physicalLocation"]["region"]["startColumn"];

            let column = if column.is_null() {
                1
            } else {
                column.as_i64().unwrap() as usize
            };

            alerts.push(Alert {
                file: artifact_uri,
                line: line_number,
                column: column,
                message: message.to_string(),
            });
        }
    }

    alerts
}

struct SourceCode {
    file_path: String,
    language: Language,
    source_code: String,
    tree: Option<Tree>,
}

impl SourceCode {
    pub fn new<'a>(file_path: &'a str) -> SourceCode {
        let mut source_code = SourceCode {
            file_path: file_path.to_string(),
            language: unsafe { tree_sitter_c() },
            source_code: String::new(),
            tree: None,
        };

        source_code.load_source_code();
        source_code
    }

    fn load_source_code(&mut self) {
        let path = Path::new(&self.file_path);

        let mut buf_reader = BufReader::new(File::open(&path).unwrap());
        let mut line = String::new();
        while buf_reader.read_line(&mut line).unwrap() > 0 {
            self.source_code.push_str(line.as_str());
            line.clear();
        }

        let mut parser = Parser::new();
        match parser.set_language(self.language) {
            Ok(_) => {
                self.tree = Some(parser.parse(self.source_code.as_str(), None)).unwrap();
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    pub fn get_node_by_line_and_offset(&self, line_number: usize, offset: usize) -> Option<Node> {
        let node = self.tree.as_ref().unwrap().root_node();

        let mut cursor = node.walk();

        let children = node.children(&mut cursor);

        for child in children {
            let found =
                self.recursive_find_node_by_line_and_offset(child, line_number, offset, None);
            if found.is_some() {
                return found;
            }
        }

        return None;
    }

    fn recursive_find_node_by_line_and_offset<'a>(
        &self,
        node: Node<'a>,
        line_number: usize,
        offset: usize,
        mut found_node: Option<Node<'a>>,
    ) -> Option<Node<'a>> {
        if found_node.is_some() {
            return found_node;
        }

        if node.start_position().row == line_number
            && node.start_position().column == offset
            && node.end_position().column >= offset
            && node.kind() == "identifier"
        {
            found_node = Some(node);
            return found_node;
        }

        if offset == 1 && node.start_position().row == line_number && node.kind() == "identifier" {
            return found_node;
        }

        if node.children(&mut node.walk()).len() > 0 {
            for n in node.children(&mut node.walk()) {
                found_node =
                    self.recursive_find_node_by_line_and_offset(n, line_number, offset, found_node);
                if found_node.is_some() {
                    return found_node;
                }
            }
        } else {
            for n in node.next_sibling() {
                found_node =
                    self.recursive_find_node_by_line_and_offset(n, line_number, offset, found_node);
                if found_node.is_some() {
                    return found_node;
                }
            }
        }

        found_node
    }

    pub fn get_parent_function_node_lines(&self, node: Node) -> (usize, usize) {
        if node.parent().is_some() {
            let parent = node.parent().unwrap();
            if parent.kind() == "function_definition" {
                return (parent.start_position().row, parent.end_position().row);
            } else {
                return self.get_parent_function_node_lines(parent);
            }
        } else {
            return (0, 0);
        }
    }

    pub fn print_function_with_node_by_line_and_offset(
        &self,
        line_number: usize,
        offset: usize,
        message: &str,
    ) {
        let node = self.get_node_by_line_and_offset(line_number, offset);
        if node.is_some() {
            let node = node.unwrap();
            let (start_line, mut end_line) = self.get_parent_function_node_lines(node);
            end_line = end_line + 1;
            for i in start_line..end_line {
                if i == line_number {
                    let line = self
                        .source_code
                        .as_str()
                        .lines()
                        .nth(i)
                        .unwrap()
                        .to_string();
                    let node_start_column = node.start_position().column;
                    let node_end_column = node.end_position().column;
                    let node_name =
                        line.as_str().as_bytes()[node_start_column..node_end_column].to_vec();
                    let node_name = String::from_utf8(node_name).unwrap();
                    let node_name_len = node_name.len();
                    let mut node_name_start_index = 0;
                    let mut node_name_end_index = node_name_len;
                    for (i, _c) in line.chars().enumerate() {
                        if i >= node_start_column && i < node_end_column {
                            if i == node_start_column {
                                node_name_start_index = i;
                            }
                            if i == node_end_column - 1 {
                                node_name_end_index = i + 1;
                            }
                        }
                    }

                    let mut colored_line = String::new();
                    colored_line.push_str(&line[0..node_name_start_index]);
                    colored_line.push_str(&node_name.green().bold().to_string());
                    colored_line.push_str(&line[node_name_end_index..]);
                    println!("{}: {}", i + 1, colored_line);
                    println!(
                        "{}",
                        (0..node_name_end_index + node_name_start_index - 2)
                            .map(|_| "-")
                            .collect::<String>()
                            + "^"
                    );
                    println!("{} {}", "ALERT:".red().bold(), message.red());
                    println!(
                        "{}",
                        (0..node_name_end_index + node_name_start_index - 1)
                            .map(|_| "-")
                            .collect::<String>()
                    );
                } else {
                    println!(
                        "{}: {}",
                        i + 1,
                        self.source_code.as_str().lines().nth(i).unwrap()
                    );
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} <file_path> <source_dir>", args[0]);
        return;
    }

    let alerts = parse_alerts(args[1].as_str(), args[2].as_str());

    for alert in alerts {
        let source_code = SourceCode::new(alert.file.as_str());
        println!("{}:{}:{}", alert.file.bold(), alert.line, alert.column);
        println!("{}", (0..32).map(|_| "=").collect::<String>());
        source_code.print_function_with_node_by_line_and_offset(
            alert.line - 1,
            alert.column - 1,
            alert.message.as_str(),
        );

        println!("\n");
    }
}
