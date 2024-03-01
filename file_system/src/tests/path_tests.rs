#[test]
fn test_path() {
    use crate::utils::path_handler::*;
    let path = "/a/b/c";
    let cwd = "/a/b";
    let result = absolutize_from(path, cwd);

    assert_eq!(result, "/a/b/c");
}

#[test]
fn test_path2() {
    use crate::utils::path_handler::*;
    let path = "/a/b/../c";
    let cwd = "/a/b";
    let result = absolutize_from(path, cwd);

    assert_eq!(result, "/a/c");
}
