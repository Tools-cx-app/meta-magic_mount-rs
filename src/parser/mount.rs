// Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use nom::{
    Finish, IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::space1,
    combinator::{all_consuming, map},
    sequence::delimited,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Mount { source: String, target: String },
    Ignore { source: String },
}

fn parse_path(input: &str) -> IResult<&str, String> {
    let quoted = delimited(tag("\""), take_while1(|c: char| c != '"'), tag("\""));
    let unquoted = take_while1(|c: char| !c.is_whitespace() && !c.is_control());

    map(alt((quoted, unquoted)), |s: &str| s.to_string()).parse(input)
}

fn parse_mount_inner(input: &str) -> IResult<&str, Command> {
    let (input, (_, _, source, _, target)) =
        (tag("bind"), space1, parse_path, space1, parse_path).parse(input)?;

    Ok((input, Command::Mount { source, target }))
}

fn parse_ignore_inner(input: &str) -> IResult<&str, Command> {
    let (input, (_, _, source)) = (tag("ignore"), space1, parse_path).parse(input)?;

    Ok((input, Command::Ignore { source }))
}

pub fn parse_command(input: &str) -> Option<Command> {
    let mut parser = all_consuming(alt((parse_mount_inner, parse_ignore_inner)));

    match parser.parse(input).finish() {
        Ok((_, cmd)) => Some(cmd),
        Err(e) => {
            log::error!("failed to parse custom command: {e:?}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, parse_command};

    #[test]
    fn test_mount_command() {
        let input = "bind /test /test1";
        let result = parse_command(input).unwrap();
        assert_eq!(
            result,
            Command::Mount {
                source: "/test".to_string(),
                target: "/test1".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_mount_with_quotes() {
        let input = r#"bind "/test" "/mnt/test1 test2""#;
        let cmd = parse_command(input).unwrap();

        assert_eq!(
            cmd,
            Command::Mount {
                source: "/test".to_string(),
                target: "/mnt/test1 test2".to_string()
            }
        );
    }

    #[test]
    fn test_parse_mount_mixed() {
        let input = r#"bind /test "/mnt/test1 test2""#;
        let cmd = parse_command(input).unwrap();
        assert_eq!(
            cmd,
            Command::Mount {
                source: "/test".to_string(),
                target: "/mnt/test1 test2".to_string()
            }
        );
    }

    #[test]
    fn test_ignore_command() {
        let input = "ignore /tmp/cache";
        let result = parse_command(input).unwrap();
        assert_eq!(
            result,
            Command::Ignore {
                source: "/tmp/cache".to_string(),
            }
        );
    }

    #[test]
    fn test_ignore_with_quotes() {
        let input = r#"ignore "/test1 test2""#;
        let cmd = parse_command(input).unwrap();

        assert_eq!(
            cmd,
            Command::Ignore {
                source: "/test1 test2".to_string(),
            }
        );
    }

    #[test]
    fn test_quoted_ignore() {
        let input = r#"ignore "/var/log/my app""#;
        let result = parse_command(input).unwrap();
        assert_eq!(
            result,
            Command::Ignore {
                source: "/var/log/my app".to_string(),
            }
        );
    }

    #[test]
    fn test_invalid_command() {
        let input = "delete /something";
        let result = parse_command(input);
        assert!(result.is_none());
    }
}
