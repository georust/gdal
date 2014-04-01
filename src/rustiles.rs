extern crate sync;
extern crate http;
extern crate test;

use std::vec::Vec;
use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::Writer;
use std::task;
use http::server::{Config, Server, Request, ResponseWriter};
use http::server::request::AbsolutePath;
use http::status::NotFound;
use http::headers;
use tile::spawn_tile_worker;
use workqueue::{WorkQueue, WorkQueueProxy};

#[allow(dead_code)]
mod gdal;
mod tile;
mod workqueue;



#[test]
fn test_nothing() {
    assert_eq!(1, 1);
}


#[deriving(Clone)]
struct TileServer {
    queue: WorkQueueProxy<(int, int, int), ~[u8]>,
}


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
                            let content_type = headers::content_type::MediaType {
                                type_: ~"image",
                                subtype: ~"png",
                                parameters: Vec::new(),
                            };
                            w.headers.content_type = Some(content_type);
                            let tile_png = self.queue.push((x, y, z)).recv();
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
    use std::os::args;
    let source_path = Path::new(args()[1]);
    let (queue, dispatcher) = WorkQueue::<(int, int, int), ~[u8]>();
    task::spawn(proc() { dispatcher.run(); });
    for _ in range(0, 4) {
        spawn_tile_worker(&queue, &source_path);
    }
    TileServer{queue: queue.proxy()}.serve_forever();
}


static index_html: &'static str = "<!doctype html>\
<meta charset='utf-8'>\n\
<title>RusTiles demo</title>\n\
<link rel='stylesheet' href='//cdnjs.cloudflare.com/ajax/libs/leaflet/0.7.2/leaflet.css'>\n\
<style>
html, body, #map { margin: 0; height: 100%; }
#slider { position: fixed; top: 0; right: 0; }
</style>
<div id='map'></div>
<div id='slider'><input type='range' min='0' max='100' value='50'></div>
<script src='//cdnjs.cloudflare.com/ajax/libs/jquery/2.1.0/jquery.min.js'></script>\n\
<script src='//cdnjs.cloudflare.com/ajax/libs/leaflet/0.7.2/leaflet.js'></script>\n\
<script>
var map = L.map('map').setView([40, 10], 3);
L.tileLayer('http://{s}.tile.osm.org/{z}/{x}/{y}.png', {
  attribution: '&copy; <a href=\\'http://osm.org/copyright\\'>' +
               'OpenStreetMap</a> contributors'}).addTo(map);
var nasa = L.tileLayer('/tile/{z}/{x}/{y}').addTo(map);
function updateOpacity() { nasa.setOpacity(+($('input').val()) / 100); }
$('input').change(updateOpacity); updateOpacity();
</script>
";
