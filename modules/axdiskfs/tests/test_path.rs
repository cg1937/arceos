fn split_path(path: &str) -> (&str, Option<&str>) {
    let trimmed_path = path.trim_start_matches('/');
    trimmed_path.find('/').map_or((trimmed_path, None), |n| {
        (&trimmed_path[..n], Some(&trimmed_path[n + 1..]))
    })
}

fn split_last_path(path: &str) -> (&str, Option<&str>) {
    let trimmed_path = path.trim_start_matches('/');
    trimmed_path.rfind('/').map_or((trimmed_path, None), |n| {
        (&trimmed_path[..n], Some(&trimmed_path[n + 1..]))
    })
}

#[test]
fn test_path() {
    let path = "/test_dir1/test3.txt";
    let (name, rest) = split_path(path);
    println!("name = {:?}", name);
    println!("rest = {:?}", rest);

    let (name, rest) = split_last_path(path);
    println!("name = {:?}", name);
    println!("rest = {:?}", rest);

    let path = "/dev";
    let (name, rest) = split_path(path);
    println!("name = {:?}", name);
    println!("rest = {:?}", rest);

    let path = "///very/long//.././long//./path/./test.txt";
    let (name, rest) = split_path(path);
    println!("name = {:?}", name);
    println!("rest = {:?}", rest);

    let (name, rest) = split_path(rest.unwrap());

    println!("name = {:?}", name);
    println!("rest = {:?}", rest);

    let (name, rest) = split_path(rest.unwrap());

    println!("name = {:?}", name);
    println!("rest = {:?}", rest);

    let (name, rest) = split_path(rest.unwrap());

    println!("name = {:?}", name);
    println!("rest = {:?}", rest);
}
