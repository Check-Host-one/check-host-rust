// Live smoke test: cargo run --example smoke
fn main() {
    let ch = checkhost::Client::new();
    let m = ch.dns("example.com", "A", 2, &["Asia"]).unwrap();
    assert_eq!(m["status"], "finished");
    for r in m["results"].as_array().unwrap() {
        println!("{}, {}: {}", r["probe"]["city"], r["probe"]["country"], r["result"]["answers"]);
    }
    println!("https://check-host.one/r/{}", m["resultSlug"].as_str().unwrap());
    assert_eq!(ch.host_info(Some("1.1.1.1")).unwrap()["asn"], 13335);
    match ch.get_measurement("nope") {
        Err(checkhost::Error::Api { status: 404, .. }) => {}
        other => panic!("want 404, got {other:?}"),
    }
}
