use warp::Filter;
use reqwest::{
  Client,
  Error
};
use tera::{
  Tera,
  Context
};
use base64::{
  engine::general_purpose,
  Engine as _
};
use std::env::var;

struct FSGameserverCon {
  ip: String,
  md5: String
} // This is temporary until I am satisfied with the page design.

#[tokio::main]
async fn main() {
  let tera = warp::any().map(move || Tera::new("templates/**/*").unwrap());

  let map_route = warp::path!("map")
    .and(tera.clone())
    .and_then(map_handler);

  let logger = warp::log::custom(|i| {
    println!(
      " Web::[ {} ][ {} ][ {} ][ {:?} ][ {} ]",
      i.remote_addr().map(|s| s.ip().to_string()).unwrap_or_else(|| "<unknown ip>".to_string()),
      i.status(),
      i.path(),
      i.elapsed(),
      i.user_agent().unwrap_or("<unknown agent>")
    )
  });

  let routes = map_route.with(logger);

  warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}

async fn map_handler(tera: Tera) -> Result<impl warp::Reply, warp::Rejection> {
  let mut context = Context::new();
  context.insert("title", "Field Ownership Viewer");

  let gameserver_con = FSGameserverCon {
    ip: var("FS_IP").unwrap().to_string(),
    md5: var("FS_MD5").unwrap().to_string()
  };

  let rs_fetch_fields = fetch_fields(&gameserver_con.ip, &gameserver_con.md5).await;
  let rs_fetch_map = fetch_map_overlay(&gameserver_con.ip, &gameserver_con.md5).await;

  match rs_fetch_fields {
    Ok(fields) => {
      let json = serde_json::to_string(&fields).unwrap();
      context.insert("rs_fetch_fields", &json);
    },
    Err(_) => context.insert("rs_fetch_fields", "null")
  }

  match rs_fetch_map {
    Ok(map) => {
      let map_base64 = general_purpose::STANDARD.encode(&map);
      context.insert("rs_fetch_map", &map_base64);
    },
    Err(_) => context.insert("rs_fetch_map", "null")
  }

  let rendered = tera.render("map/index.html", &context).unwrap();

  Ok(warp::reply::html(rendered))
}

async fn fetch_map_overlay(server_ip: &str, md5: &str) -> Result<Vec<u8>, Error> {
  let url = format!("http://{}/feed/dedicated-server-stats-map.jpg?code={}&quality=100&size=2048", server_ip, md5);
  let client = Client::new();
  let resp = client.get(&url).send().await?;
  let bytes = resp.bytes().await?;

  Ok(bytes.to_vec())
}

async fn fetch_fields(server_ip: &str, md5: &str) -> Result<serde_json::Value, Error> {
  let url = format!("http://{}/feed/dedicated-server-stats.json?code={}", server_ip, md5);
  let client = Client::new();
  let resp = client.get(&url).send().await?;
  let json = resp.json().await?;

  Ok(json)
}
