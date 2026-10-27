
use serde::{Deserialize,Serialize};
#[derive(Clone,Serialize,Deserialize)]
#[serde(default,deny_unknown_fields)]
pub struct Crosshair {
 pub enabled:bool,pub shape:String,pub grid:u8,pub pixels:Vec<bool>,
 pub size:f64,pub thickness:f64,pub gap:f64,pub dot:f64,pub outline:f64,pub opacity:f64,
 pub color:String,pub chroma:bool,pub dynamic:bool,pub target_color:bool,
 pub hostile_color:String,pub passive_color:String,pub range_color:String,
 pub cooldown:String,pub hitmarker:bool,pub hit_color:String,pub hit_duration:u32,
}
impl Default for Crosshair{fn default()->Self{Self{enabled:false,shape:"cross".into(),grid:15,pixels:vec![false;225],size:6.,thickness:1.5,gap:3.,dot:1.5,outline:1.,opacity:1.,color:"#FFFFFF".into(),chroma:false,dynamic:false,target_color:false,hostile_color:"#EF4444".into(),passive_color:"#10B981".into(),range_color:"#F59E0B".into(),cooldown:"off".into(),hitmarker:false,hit_color:"#D946EF".into(),hit_duration:180}}}
pub fn color(value:&str)->bool{value.len()==7&&value.starts_with('#')&&value[1..].bytes().all(|b|b.is_ascii_hexdigit())}
impl Crosshair{pub fn valid(&self)->bool{
 ["dot","cross","circle","diamond","chevron","cross-dot","pixels"].contains(&self.shape.as_str())&&[15,17].contains(&self.grid)&&self.pixels.len()==usize::from(self.grid).pow(2)
 &&[(self.size,1.,18.),(self.thickness,0.5,6.),(self.gap,0.,12.),(self.dot,0.5,6.),(self.outline,0.,3.),(self.opacity,0.1,1.)].iter().all(|(n,a,b)|n.is_finite()&&(*a..=*b).contains(n))
 &&[&self.color,&self.hostile_color,&self.passive_color,&self.range_color,&self.hit_color].iter().all(|s|color(s))&&["off","ring","bar"].contains(&self.cooldown.as_str())&&(60..=1000).contains(&self.hit_duration)
}}
#[cfg(test)]mod tests{use super::*;#[test]fn rejects_corrupt_geometry(){let mut c=Crosshair::default();assert!(c.valid());c.pixels.push(true);assert!(!c.valid());c.pixels.pop();c.size=f64::NAN;assert!(!c.valid());}#[test]fn colors_are_data_only(){assert!(color("#A855F7"));assert!(!color("url(x)"));assert!(!color("#éFFF"));}}
