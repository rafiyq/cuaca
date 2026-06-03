use super::cap::parse_cap;
use super::fetch::cap_id_from_link;
use super::polygon::parse_polygon;
use super::polygon::point_in_polygon;

#[test]
fn test_cap_id_from_link() {
    assert_eq!(
        cap_id_from_link("https://www.bmkg.go.id/alerts/nowcast/id/CBT20260520001_alert.xml"),
        "CBT20260520001"
    );
    assert_eq!(cap_id_from_link("https://example.com/foo.xml"), "foo");
}

#[test]
fn test_parse_cap_sample() {
    let sample = r#"<?xml version="1.0" ?>
<alert xmlns="urn:oasis:names:tc:emergency:cap:1.2">
  <identifier>2.49.0.1.360.0.2026.05.20.01.36.001</identifier>
  <sender>cuaca.ekstrem@bmkg.go.id</sender>
  <sent>2026-05-20T07:55:00+07:00</sent>
  <status>Actual</status>
  <msgType>Alert</msgType>
  <scope>Public</scope>
  <info>
    <language>id</language>
    <category>Met</category>
    <event>Hujan Lebat dan Petir</event>
    <urgency>Immediate</urgency>
    <severity>Moderate</severity>
    <certainty>Observed</certainty>
    <eventCode>
      <valueName>OET:v1.2</valueName>
      <value>OET-194</value>
    </eventCode>
    <effective>2026-05-20T08:05:00+07:00</effective>
    <expires>2026-05-20T10:00:00+07:00</expires>
    <senderName>Badan Meteorologi Klimatologi dan Geofisika</senderName>
    <headline>Hujan Lebat disertai Petir di Banten</headline>
    <description>Hujan lebat...</description>
    <web>https://nowcasting.bmkg.go.id/infografis/CBT/2026/05/20/infografis.jpg</web>
    <contact>06221 196</contact>
    <area>
      <areaDesc>Banten</areaDesc>
      <polygon>-6.024,106.412 -6.031,106.408 -6.030,106.384</polygon>
    </area>
  </info>
</alert>"#;
    let alert = parse_cap(sample.as_bytes()).expect("parse cap");
    assert_eq!(alert.info.headline, "Hujan Lebat disertai Petir di Banten");
    assert_eq!(alert.info.area.area_desc, "Banten");
    assert!(!alert.info.effective.is_empty());
    assert!(!alert.info.expires.is_empty());
    assert!(!alert.info.area.polygons.is_empty());
    assert!(alert.info.web.is_some());
}

#[test]
fn test_point_in_polygon() {
    let square = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
    assert!(point_in_polygon((5.0, 5.0), &square));
    assert!(!point_in_polygon((15.0, 5.0), &square));
}

#[test]
fn test_parse_polygon() {
    let s = " -6.024,106.412 -6.031,106.408 -6.030,106.384 ";
    let pts = parse_polygon(s);
    assert_eq!(pts.len(), 3);
    assert_eq!(pts[0], (-6.024, 106.412));
}
