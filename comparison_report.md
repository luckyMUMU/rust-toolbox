# 分类效果对比报告

## 1. 概览
- **总文件夹数 (Common)**: 1468
- **完全匹配**: 735 (50.07%)
- **不匹配**: 733
- **Rust 未分类**: 278

## 2. 差异分析
Rust 工具由于为了避免崩溃而进行了关键词去重（全局唯一），导致部分规则失效。Python 脚本支持复杂的关键词组合和重复关键词（多分类匹配）。

### 差异示例 (前20个)
| 文件夹 | Rust 结果 | Python 结果 |
|---|---|---|
| Coser @ 抖 娘 - 利 世： 下 半球 (32 photos) - ( Page 1 ／ 2 ) | Unclassified | China@抖娘利世 |
| Coser @ 抖 娘 - 利 世： 水蓝 和服 (30 photos) - ( Page 1 ／ 2 ) | Unclassified | China@抖娘利世 |
| Jeong Jenny 정제니, BLUECAKE Vol.10 “Sleeping with Ganyu” Set.03 | Ambiguous | Korea@Jenn_젠 |
| GIRLT No.042： Model Aojiao Meng Meng (K8 傲 娇 萌萌 Vivian) (54 photos) - ( Page 1 ／ 3 ) | Unclassified | China@laura阿姣 |
| Jeong Jenny 정제니, [DJAWA] A Drowsy Day Set.01 | Ambiguous | Korea@Jenn_젠 |
| Coser @ 过期 米线 线 喵： 透明 围裙 会员 版 (33 photos) - ( Page 1 ／ 2 ) | Unclassified | China@过期米线喵喵 |
| JangJoo 장주, ArtGravia Vol.295 Photobook Set.02 | Ambiguous | China@阿色 |
| Jungmi 정미, [LimePunch] Vol.2 Relaxation Set.01 | Unclassified | China@阿色 |
| Cosplay 雯妹不讲道理 大连旅拍1 连体泳装 Set.01 | China@雯妹不讲道理 | China@阿色 |
| Booty Queen, [BLUECAKE] Rabbit Pink RED+ | Ambiguous | Korea@Booty_Queen |
| Coser @ 过期 米线 线 喵： 马戏团 (22 photos) | Unclassified | China@过期米线喵喵 |
| CRAZYGIANT Hanere - Sexy Photobook (55P) | Unclassified | China@阿色 |
| Coco 수민, [Patreon] Vol.9 Girl Friend Set.02 | Org@Patreon | China@阿色 |
| Booty Queen, [Bimilstory] Gal in a Shool Warehouse Set.01 | Ambiguous | Korea@Booty_Queen |
| Coco 수민, [Patreon] Give Me A Big Carrot Set.02 | Org@Patreon | China@阿色 |
| Jeong Jenny 정제니, DJAWA ‘Sweet Talk Chobits’ Set.02 | Ambiguous | Korea@Jenn_젠 |
| Jeong Jenny 정제니, BLUECAKE ‘Kurumi Bunny’ Set.01 | Ambiguous | Korea@Jenn_젠 |
| Jeong Jenny 정제니, [BLUECAKE] BLUISH Set.01 | Ambiguous | Korea@Jenn_젠 |
| JangJoo 장주, ArtGravia Vol.207 Photobook Set.02 | Ambiguous | China@阿色 |
| EunWoo 은우, Loozy 「New Girl」 Set.01 | Ambiguous | Korea@Woo |
