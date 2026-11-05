const keys = ['Explorar launcher', 'Puedes explorar el launcher. Para iniciar Minecraft debes verificar una cuenta con licencia.'] as const;
const values: Record<string, [string, string]> = {
  'en-US': ['Explore launcher', 'You can explore the launcher. To start Minecraft, verify an account with a game license.'],
  'pt-BR': ['Explorar launcher', 'Você pode explorar o launcher. Para iniciar o Minecraft, verifique uma conta com licença do jogo.'],
  'fr-FR': ['Explorer le launcher', 'Vous pouvez explorer le launcher. Pour lancer Minecraft, vérifiez un compte possédant une licence du jeu.'],
  'de-DE': ['Launcher erkunden', 'Du kannst den Launcher erkunden. Um Minecraft zu starten, bestätige ein Konto mit einer Spiellizenz.'],
  'it-IT': ['Esplora il launcher', 'Puoi esplorare il launcher. Per avviare Minecraft, verifica un account con una licenza del gioco.'],
  'ru-RU': ['Открыть лаунчер', 'Вы можете просматривать лаунчер. Чтобы запустить Minecraft, подтвердите аккаунт с лицензией игры.'],
  'zh-CN': ['浏览启动器', '你可以浏览启动器。启动 Minecraft 前，请验证拥有游戏许可的账户。'],
  'ja-JP': ['ランチャーを見る', 'ランチャーを閲覧できます。Minecraft を起動するには、ゲームのライセンスを持つアカウントを認証してください。'],
  'ko-KR': ['런처 둘러보기', '런처를 둘러볼 수 있습니다. Minecraft를 실행하려면 게임 라이선스가 있는 계정을 인증하세요.'],
  'ca-ES': ['Explora el launcher', 'Pots explorar el launcher. Per iniciar Minecraft, verifica un compte amb llicència del joc.'],
  'eo-UY': ['Esplori la lanĉilon', 'Vi povas esplori la lanĉilon. Por lanĉi Minecraft, kontrolu konton kun ludlicenco.']
};
export default Object.fromEntries(Object.entries(values).map(([code, translations]) => [code, Object.fromEntries(keys.map((key, i) => [key, translations[i]]))]));
