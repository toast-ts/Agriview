use warp::Filter;
use tokio::time::{
  Duration,
  sleep
};
use std::{
  env::var,
  sync::{
    Arc,
    Mutex
  }
};
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

struct FSGameserverCon {
  ip: String,
  md5: String
} // This is temporary until I am satisfied with the page design.

struct MemoryCache {
  fields: Option<serde_json::Value>,
  map: Option<Vec<u8>>
}

impl MemoryCache {
  async fn update(&mut self, ip: &str, md5: &str) {
    self.fields = fetch_fields(ip, md5).await.ok();
    self.map = fetch_map_overlay(ip, md5).await.ok();
  }
}

async fn periodic_cache_update(cache: Arc<Mutex<MemoryCache>>, ip: &str, md5: &str) {
  loop {
    sleep(Duration::from_secs(60)).await;
    let mut cache = cache.lock().unwrap();
    cache.update(ip, md5).await;
    println!(" Web::[ In-memory cache updated ]")
  }
}

#[tokio::main]
async fn main() {
  let gameserver_con = FSGameserverCon {
    ip: var("FS_IP").unwrap().to_string(),
    md5: var("FS_MD5").unwrap().to_string()
  };
  let cache = Arc::new(Mutex::new(MemoryCache {
    fields: None,
    map: None
  }));

  {
    let mut cache = cache.lock().unwrap();
    cache.update(&gameserver_con.ip, &gameserver_con.md5).await;
    println!(" Web::[ Initial in-memory cache populated ]")
  }

  let cache_clone = Arc::clone(&cache);
  tokio::task::spawn_blocking(move || {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
      periodic_cache_update(cache_clone, &gameserver_con.ip, &gameserver_con.md5).await;
    });
  });

  let tera = warp::any().map(move || Tera::new("templates/**/*").unwrap());
  let cache_filter = warp::any().map(move || Arc::clone(&cache));

  let map_route = warp::path!("map")
    .and(tera.clone())
    .and(cache_filter.clone())
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

async fn map_handler(tera: Tera, cache: Arc<Mutex<MemoryCache>>) -> Result<impl warp::Reply, warp::Rejection> {
  let mut context = Context::new();
  context.insert("title", "Agriview");

  let cache = cache.lock().unwrap();

  match &cache.fields {
    Some(fields) => {
      let json = serde_json::to_string(fields).unwrap();
      context.insert("rs_fetch_fields", &json);
    },
    None => context.insert("rs_fetch_fields", "null")
  }

  match &cache.map {
    Some(map) => {
      let map_base64 = general_purpose::STANDARD.encode(map);
      context.insert("rs_fetch_map", &map_base64);
    },
    None => context.insert("rs_fetch_map", "null")
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
  let mut json: serde_json::Value = resp.json().await?;
  let fields = json["fields"].take(); // We only want the fields object and not the entire JSON data from the gameserver

  Ok(fields)
}
