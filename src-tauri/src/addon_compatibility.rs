
pub const HIT_COLOR_263_SHA1: &str = "ebfe5a8f1aec0d0f96f5b1a3176eff74bc70992d";

pub fn blocked(game: &str, loader: &str, module: &str, sha1: &str) -> bool {
    game == "26.3"
        && matches!(loader, "fabric" | "quilt")
        && module == "hit_color"
        && sha1.eq_ignore_ascii_case(HIT_COLOR_263_SHA1)
}

pub fn reason() -> String {
    "Hit Color fabric-26.3-0.1 se ha pausado para esta partida: modifica los shaders de armadura y provoca el error Sampler1 en Minecraft 26.3. El archivo se conserva y el HUD sigue disponible.".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_the_identified_artifact_and_runtime_are_blocked() {
        assert!(blocked("26.3", "fabric", "hit_color", HIT_COLOR_263_SHA1));
        assert!(blocked("26.3", "quilt", "hit_color", &HIT_COLOR_263_SHA1.to_uppercase()));
        assert!(!blocked("26.2", "fabric", "hit_color", HIT_COLOR_263_SHA1));
        assert!(!blocked("26.3", "forge", "hit_color", HIT_COLOR_263_SHA1));
        assert!(!blocked("26.3", "fabric", "iris", HIT_COLOR_263_SHA1));
        assert!(!blocked("26.3", "fabric", "hit_color", &"a".repeat(40)));
    }
}
