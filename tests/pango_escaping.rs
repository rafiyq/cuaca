use cuaca::util::escape_pango;

#[test]
fn test_escape_pango() {
    assert_eq!(escape_pango("Hello & World"), "Hello &amp; World");
    assert_eq!(escape_pango("3 < 4 > 2"), "3 &lt; 4 &gt; 2");
    assert_eq!(escape_pango("a & b < c > d"), "a &amp; b &lt; c &gt; d");
    assert_eq!(escape_pango("no special"), "no special");
}
