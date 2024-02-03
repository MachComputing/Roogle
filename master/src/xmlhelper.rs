use std::fs::File;
use std::io::BufReader;
use xml::reader::XmlEvent;
use xml::EventReader;

pub fn recursive_ev_find(
    ev_reader: &mut EventReader<BufReader<File>>,
    path: &[&str],
) -> Option<String> {
    let mut buffer: String = String::new();
    let mut add = false;

    loop {
        match ev_reader.next() {
            Ok(XmlEvent::StartElement { name: n, .. }) => {
                if n.local_name == *path[0] && path.len() > 1 {
                    return recursive_ev_find(ev_reader, &path[1..]);
                }

                if n.local_name == *path[0] && path.len() == 1 {
                    add = true;
                }
            }
            Ok(XmlEvent::Characters(chars)) => {
                if add {
                    buffer.push_str(&chars);
                    break;
                }
            }
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error: {}", err);
                return None;
            }
        }
    }

    Some(buffer)
}
