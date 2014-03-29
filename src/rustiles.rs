extern crate sync;
extern crate http;
extern crate geom;

use std::vec::Vec;
use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::Writer;
use http::server::{Config, Server, Request, ResponseWriter};
use http::server::request::AbsolutePath;
use http::status::NotFound;
use http::headers;
use tile::tile;

#[allow(dead_code)]
mod gdal;
mod tile;



#[test]
fn test_nothing() {
    assert_eq!(1, 1);
}


#[deriving(Clone)]
struct TileServer;


impl Server for TileServer {
    fn get_config(&self) -> Config {
        Config {
            bind_address: SocketAddr {
                ip: Ipv4Addr(0, 0, 0, 0),
                port: 8001,
            },
        }
    }

    fn handle_request(&self, r: &Request, w: &mut ResponseWriter) {
        w.headers.content_type = Some(headers::content_type::MediaType {
            type_: ~"text",
            subtype: ~"html",
            parameters: Vec::new(),
        });

        match r.request_uri {
            AbsolutePath(ref url) => {
                if url == &~"/" {
                    w.write(index_html.as_bytes()).unwrap();
                    return;
                }
                let bits: ~[&str] = url.split('/').collect();
                if bits.len() == 5 && bits[0] == "" && bits[1] == "tile" {
                    match (
                        from_str::<int>(bits[2]),
                        from_str::<int>(bits[3]),
                        from_str::<int>(bits[4])
                    ) {
                        (Some(z), Some(x), Some(y)) => {
                            use std::os::args;
                            use std::path::Path;
                            use gdal::dataset::open;
                            let content_type = headers::content_type::MediaType {
                                type_: ~"image",
                                subtype: ~"png",
                                parameters: Vec::new(),
                            };
                            let source = open(&Path::new(args()[1])).unwrap();
                            w.headers.content_type = Some(content_type);
                            let tile_png = tile(source, (x, y, z));
                            w.write(tile_png).unwrap();
                        },
                        _ => {}
                    }
                }
            },
            _ => {}
        };

        w.status = NotFound;
        w.write("Page not found :(\n".as_bytes()).unwrap();

    }
}


fn main() {
    TileServer.serve_forever();
}


static index_html: &'static str = "<!doctype html>\
<meta charset='utf-8'>\n\
<title>RusTiles demo</title>\n\
<link rel='stylesheet' href='//cdnjs.cloudflare.com/ajax/libs/leaflet/0.7.2/leaflet.css'>\n\
<style>
html, body, #map { margin: 0; height: 100%; }
</style>
<div id='map'></div>
<script src='//cdnjs.cloudflare.com/ajax/libs/leaflet/0.7.2/leaflet.js'></script>\n\
<script>
var map = L.map('map').setView([40, 10], 3);
L.tileLayer('/tile/{z}/{x}/{y}').addTo(map);
</script>
";
