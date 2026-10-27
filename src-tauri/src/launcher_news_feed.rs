
use super::NewsItem;
use serde_json::Value;
fn image(entry:&Value,field:&str)->Option<String>{
 let raw=entry.get(field)?.get("url")?.as_str()?.trim();if raw.is_empty(){return None;}
 let address=url::Url::parse("https://launchercontent.mojang.com/").ok()?.join(raw).ok()?;
 (address.scheme()=="https"&&address.username().is_empty()&&address.password().is_none()).then(||address.to_string())
}
pub(super) fn parse(feed:&Value)->Vec<NewsItem>{
 let mut rows=Vec::new();for entry in feed["entries"].as_array().into_iter().flatten(){
  let Some(url)=entry["readMoreLink"].as_str().and_then(|s|url::Url::parse(s).ok()).filter(|u|u.scheme()=="https"&&u.username().is_empty()&&u.password().is_none()&&u.host_str().is_some_and(|h|h=="minecraft.net"||h.ends_with(".minecraft.net")))else{continue};
  let play=image(entry,"playPageImage");let news=image(entry,"newsPageImage");
  rows.push(NewsItem{title:entry["title"].as_str().unwrap_or("Minecraft").chars().take(200).collect(),summary:entry["text"].as_str().unwrap_or("").chars().take(500).collect(),image:play.clone().or_else(||news.clone()).unwrap_or_default(),image_fallback:news.filter(|url|Some(url)!=play.as_ref()).unwrap_or_default(),url:url.to_string(),date:entry["date"].as_str().unwrap_or("").into(),category:entry["category"].as_str().unwrap_or("Minecraft").into()});
 }rows.sort_by(|a,b|b.date.cmp(&a.date));rows.truncate(9);rows
}
#[cfg(test)]mod tests{
 use super::*;use serde_json::json;
 fn entry()->Value{json!({"title":"Test","readMoreLink":"https://www.minecraft.net/article/test","date":"2026-10-26","playPageImage":{"url":"/images/play.png"},"newsPageImage":{"url":"https://launchercontent.mojang.com/images/news.jpeg"}})}
 #[test]fn remote_feed_images_are_resolved_without_copying_assets(){let rows=parse(&json!({"entries":[entry()]}));assert_eq!(rows[0].image,"https://launchercontent.mojang.com/images/play.png");assert_eq!(rows[0].image_fallback,"https://launchercontent.mojang.com/images/news.jpeg");}
 #[test]fn empty_or_unsafe_primary_uses_the_real_secondary(){for raw in ["","javascript:alert(1)","data:image/png,invalid","https://user:pass@example.com/a.png"]{let mut e=entry();e["playPageImage"]["url"]=json!(raw);let rows=parse(&json!({"entries":[e]}));assert_eq!(rows[0].image,"https://launchercontent.mojang.com/images/news.jpeg");assert!(!rows[0].image.ends_with(".com/"));}}
 #[test]fn news_links_cannot_escape_the_official_article_site(){let mut e=entry();e["readMoreLink"]=json!("https://minecraft.net.attacker.invalid/");assert!(parse(&json!({"entries":[e]})).is_empty());}
}
