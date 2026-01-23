# GitHub Copilot Code Review (2026/01/21) - アクション一覧

## 概要

このファイルは、コードレビューで指摘された問題に対する実装タスク（アクション）を管理します。
詳細な分析・調査結果は [`response.md`](./response.md) を参照してください。

## ステータス凡例

- `[ ]` 未着手
- `[進]` 作業中
- `[✓]` 完了
- `[保]` 保留

---

## アクション一覧

| ID       | 優先度 | 依存               | ステータス | 概要                                   | 対応元                           |
| -------- | ------ | ------------------ | ---------- | -------------------------------------- | -------------------------------- |
| ACTION-1 | 高     | -                  | [✓]        | Pure Data Structure 化（テスト追加含） | 問題2-1, 問題3-2, 問題5-1, 懸念2 |
| ACTION-2 | 高     | -                  | [✓]        | ベンチマークの追加                     | 問題1-1                          |
| ACTION-3 | 中     | ACTION-1, ACTION-2 | [✓]        | 双方向マッピングの実装（実施不要）     | 問題1-1                          |
| ACTION-4 | 中     | -                  | [✓]        | ドキュメント整備とコード改善           | 問題1-3, 問題3-1, 懸念1, 懸念3   |
| ACTION-5 | 低     | -                  | [✓]        | Box::leak 修正                         | 問題2-2                          |
| ACTION-6 | 中     | -                  | [✓]        | check_consistency API への置き換え     | 問題4-2                          |
| ACTION-7 | 低     | -                  | [✓]        | BacktrackSolver のテスト調査           | 問題5-2                          |

---

## ACTION-1: Pure Data Structure 化

- **優先度**: 高（ブロッカー）
- **依存**: なし
- **ステータス**: [✓]
- **作業量**: 大
- **対応元**: 問題2-1, 懸念2

### Pure Data Structure 化とは

`CandidateGrid` から Sudoku のルール（制約伝播）を削除し、単なる「候補の状態管理」のみを行うデータ構造にする。
制約伝播は **Naked Single technique** に組み込む。

### 設計の根拠

**なぜ Naked Single に制約伝播を組み込むか**:

1. **TechniqueSolver の reset 戦略**:
   - 任意の technique で変更があると、最初の technique（Naked Single）に戻る
   - HiddenSingle で配置 → NakedSingle が実行 → 制約伝播が自動的に実行される
   - すべての technique の後に Naked Single が実行されることが保証される

2. **Naked Single は fundamental technique**:
   - 実用的なすべての Sudoku solver に含まれる
   - 「確定セルを見つけ、その結果（制約伝播）を反映する」基盤 technique
   - 他の technique は「どのセルが確定するか」を見つけるだけ

3. **実装がシンプル**:
   - 独立した `ConstraintPropagation` technique が不要
   - technique リストの重複がない

### 作業内容

1. **`CandidateGrid::place` から制約伝播を削除**
   - ファイル: `crates/sudoku-core/src/candidate_grid.rs`
   - 変更: セル自身の候補削除のみを行う（他のセルに影響しない）
   - メソッド名は `place` のまま維持

2. **Naked Single に制約伝播を組み込む**
   - ファイル: `crates/sudoku-solver/src/technique/naked_single.rs`
   - 変更: 確定セル（候補が1つ）を検出し、以下を実行:
     1. `grid.place(pos, digit)` で配置（Pure 化後は制約伝播なし）
     2. 手動で row/col/box から `digit` を除外

     ```rust
        for row_pos in Position::ROWS[pos.y()] {
            if row_pos != pos {
                grid.remove_candidate(row_pos, digit);
            }
        }
        // col, box も同様
     ```

3. **`place_no_propagation` を削除**
   - Pure 化により不要になる

4. **テストの修正**
   - `place_no_propagation` の使用箇所を `place` に書き換え
   - Naked Single のテストを更新（制約伝播を検証）

### 影響範囲

- NakedSingle の実装変更を含む（問題3-2に対応）
- テストコードの `place_no_propagation` 使用箇所を修正（問題5-1に対応）
- ACTION-3 の実装が容易になる（双方向マッピングの同期処理がシンプルになる）

### チェックリスト

- [x] `CandidateGrid::place` の実装変更（制約伝播の削除）
- [x] `NakedSingle::apply` の実装変更（制約伝播の追加）
- [x] `place_no_propagation` の削除
- [x] `CandidateGrid` のテスト更新
- [x] `NakedSingle` のテスト更新（制約伝播の検証）
- [x] 既存の統合テストが通ることを確認
- [x] ドキュメントの更新（Pure Data Structure の説明）
- [x] `BacktrackSolver::pure_backtrack()` を `without_techniques()` にリネーム（"pure" 用語の混同を回避）
- [x] `docs/ARCHITECTURE.md` の更新（constraint propagation 関連の記述を修正）

---

## ACTION-2: ベンチマークの追加

- **優先度**: 高
- **依存**: なし
- **ステータス**: [進行中] 1/4 完了
- **作業量**: 中
- **対応元**: 問題1-1

### 背景

#### レビュー指摘の内容

レビューでは digit-centric 設計に対して以下の性能問題が指摘された：

1. **`candidates_at` の性能**: position → candidates のクエリが O(9) の線形走査
2. **`classify_cells` の性能**: 全セルの候補数計算で 81×9 回のチェックが発生
3. **`decided_cells` の性能**: `classify_cells` を経由するため同様に非効率

#### 実装調査の結果

**`candidates_at` の呼び出し箇所**:

- **実際の呼び出し**: `find_best_assumption` のみ（バックトラック探索の各ステップで1回）
- **最適化済みの箇所**: `to_digit_grid` はコミット `5cdd9302` で `candidates_at` を使わない実装に最適化済み

**cell-oriented で高速化される可能性がある処理**:

`CandidateGrid` の全公開メソッドを調査した結果：

- **cell-oriented かつ読み取り専用のメソッド**: `candidates_at` のみ
  - Position を引数に取り、position → candidates の情報を返す（`&self`）
  - 双方向マッピングで O(9) → O(1) に改善可能
- **書き込み操作**（`place`, `remove_candidate`）: 双方向マッピングでは同期が必要なため、処理量が増えるだけ
- **digit-oriented なメソッド**（`digit_positions`, `row_mask` など）: Digit を引数に取るため対象外

### 前提条件

- **リファクタリング**: `find_best_assumption` の共通化
  - `sudoku-generator/src/lib.rs` と `sudoku-solver/src/backtrack_solver.rs` に同じ実装が存在
  - ベンチマーク追加前に共通化が必要（共通化方法は要検討）

### 検討事項と決定事項

#### 1. ベンチマークフレームワークの選定

**決定**: **Criterion.rs** を使用

**理由**:

- Rust エコシステムで最も広く使われている（tokio, serde, regex など）
- 統計的分析が充実（平均、中央値、標準偏差、外れ値検出）
- HTMLレポート生成（グラフ付き）
- ベースラインとの比較機能、回帰検出
- stable Rust で動作
- 将来的に CI での追跡も可能

**トレードオフ**: 実行時間が長い（統計的信頼性のため）が、10%の改善を検出するには必要な精度

#### 2. `find_best_assumption` の共通化方法

**決定**: `sudoku-solver` クレートに新しい `backtrack` モジュールを作成し、公開する

**配置**:

- ファイル: `sudoku-solver/src/backtrack.rs` (新規作成)
- 公開: `pub mod backtrack` として公開（re-export なし）
- 関数名: `find_best_assumption` (元の名前を維持)
- 使用例: `sudoku_solver::backtrack::find_best_assumption(&grid)`

**理由**:

- `sudoku-generator` は既に `sudoku-solver` に依存している
- バックトラック探索のヒューリスティックは「解決アルゴリズム」の一部
- モジュール名前空間により、関数名の文脈が明確になる
- 将来的に他のバックトラック関連ユーティリティも追加可能

#### 3. ベンチマーク対象の選定原則

**決定**: 最初は最小限（ACTION-3 判断用のみ）に集中

**測定する価値がある操作**:

- 複雑な計算を含む（ループ、条件分岐、ビット演算など）
- 頻繁に呼ばれる（ホットパスにある操作）
- 最適化の余地がある
- パフォーマンスが重要

**測定不要な操作**:

- 単純なメモリ確保と初期化（例: `CandidateGrid::new`）
- トリビアルなゲッター
- コンパイラが完全に最適化できる操作

**ACTION-3 判断用の測定項目**:

- ~~`candidates_at` 単体~~（スキップ: `find_best_assumption` で間接的に測定される）
- `find_best_assumption` 単体（実際の使用パターン）
- エンドツーエンド（パズル生成・解決）
  - バックトラック回数も記録（性能への寄与を分析）

#### 4. ベンチマークのクレート配置

**決定**: 各クレートの `benches/` ディレクトリに配置

**配置例**:

- `sudoku-solver/benches/backtrack.rs` - `find_best_assumption` のベンチマーク
- `sudoku-solver/benches/solver.rs` - `BacktrackSolver::solve` のベンチマーク
- `sudoku-generator/benches/generator.rs` - パズル生成のベンチマーク

**理由**:

- Rust の標準的なプラクティス
- Criterion.rs の推奨構成
- 各クレートの public API を測定（内部 API へのアクセスは不要）

#### 5. テストデータの準備方法

**決定**: `PuzzleGenerator` で生成したパズルを使用（seed 固定）

**理由**:

- 現在のソルバー（NakedSingle + HiddenSingle のみ）では難問が解けない
- 生成したパズルは現在のテクニックで確実に解ける
- seed を固定することで再現性を確保
- 将来テクニックが増えても、ベンチマークは安定

**手順**:

1. `PuzzleGenerator` でパズルを数個生成
2. seed を記録
3. ベンチマークでは seed から再生成して使用

#### 6. テクニックセットの固定

**決定**: `fundamental_techniques()` 関数を追加

**実装**:

```rust
// sudoku-solver/src/technique/mod.rs

/// Returns the fundamental techniques (current baseline).
///
/// This function provides a stable set of techniques for benchmarking purposes.
/// The techniques included here represent the baseline implementation and should
/// not change when new techniques are added.
///
/// Current techniques: Naked Single, Hidden Single
pub fn fundamental_techniques() -> Vec<BoxedTechnique> {
    vec![Box::new(NakedSingle::new()), Box::new(HiddenSingle::new())]
}

/// Returns all available techniques.
///
/// Techniques are ordered from easiest to hardest.
/// This list may grow as new techniques are implemented.
pub fn all_techniques() -> Vec<BoxedTechnique> {
    fundamental_techniques()
    // Future: add more techniques here
}
```

**ソルバーのコンストラクタ追加**:

```rust
// TechniqueSolver, BacktrackSolver に追加
pub fn with_fundamental_techniques() -> Self {
    Self::with_techniques(technique::fundamental_techniques())
}
```

**ベンチマークでの使用**:

```rust
let solver = BacktrackSolver::with_fundamental_techniques();
```

**理由**:

- 将来テクニックが追加されても、ベンチマーク結果の意味が変わらない
- 性能比較のベースラインが安定
- `all_techniques()` は将来の拡張に対応

### 作業内容

#### 1. リファクタリング（前提作業）

1. `sudoku-solver/src/backtrack.rs` を新規作成
2. `find_best_assumption` を実装
3. `sudoku-solver/src/lib.rs` で `pub mod backtrack` として公開
4. `sudoku-generator` と `sudoku-solver` の既存コードを書き換え

#### 2. テクニックセットの固定

1. `fundamental_techniques()` 関数を追加
2. `TechniqueSolver::with_fundamental_techniques()` を追加
3. `BacktrackSolver::with_fundamental_techniques()` を追加

#### 3. ベンチマーク環境のセットアップ

1. Criterion.rs を `workspace.dependencies` と各クレートの `dev-dependencies` に追加
2. テストデータを準備
   - `with_fundamental_techniques()` でパズルを生成
   - seed を記録（ドキュメント or コード内）

#### 4. ベンチマーク実装

ACTION-3 判断用のベンチマークを実装：

1. **`find_best_assumption` 単体** (`sudoku-solver/benches/backtrack.rs`)
   - 盤面の埋まり具合を変えて測定（空 / 部分的 / ほぼ満杯）
2. **エンドツーエンド**
   - パズル生成 (`sudoku-generator/benches/generator.rs`)
   - パズル解決 (`sudoku-solver/benches/solver.rs`)
   - バックトラック回数も記録

### 判断基準

ベンチマーク結果を見て、ACTION-3 (双方向マッピング) の要否を判断：

- **判断観点**: `candidates_at` の呼び出しコストがボトルネックになっているか
- **判断基準**:
  - エンドツーエンド（パズル生成・解決）で **10% 以上の改善**が見込めれば ACTION-3 を実施
  - 改善幅が小さい場合（< 10%）は実装コストに見合わないためスキップ
- **補助指標**:
  - バックトラック回数: `find_best_assumption` の呼び出し頻度
  - `find_best_assumption` 単体の性能: 改善の上限値
- **考慮事項**:
  - メモリ増加: +91バイト（`Array81<DigitSet>` = 81 × 9bit）
  - コード複雑度: `place`/`remove_candidate` で双方向同期が必要
  - メンテナンス: 両データ構造の整合性を保つ必要

### チェックリスト

#### 1. リファクタリング ✅ **完了** (2026-01-23)

- [x] `sudoku-solver/src/backtrack.rs` を新規作成
- [x] `backtrack::find_best_assumption` を実装
- [x] `sudoku-solver/src/lib.rs` で公開
- [x] `sudoku-generator` の既存コードを書き換え
- [x] `sudoku-solver/src/backtrack_solver.rs` の既存コードを書き換え
- [x] モジュールドキュメントを追加（MRV ヒューリスティックの説明）
- [x] 5つのテストケースを実装（正常系、パニック系、検証系）
- [x] `Position::ALL` イテレーションパターンを統一（`DigitGrid::fmt` 修正）

#### 2. テクニックセットの固定 ✅ **完了** (2026-01-23)

- [x] `technique::fundamental_techniques()` を実装
- [x] `technique::all_techniques()` を更新（`fundamental_techniques()` を呼び出すように変更）
- [x] `TechniqueSolver::with_fundamental_techniques()` を追加
- [x] `BacktrackSolver::with_fundamental_techniques()` を追加

#### 3. ベンチマーク環境のセットアップ ✅ **完了** (2026-01-23, commit 745d274)

- [x] Criterion.rs を依存関係に追加
- [x] テストデータを準備（パズル生成 + seed 記録）
- [x] 各クレートに `benches/` ディレクトリを作成

#### 4. ベンチマーク実装 ✅ **完了** (2026-01-23, commit 745d274)

- [x] `sudoku-solver/benches/backtrack.rs` を実装
  - [x] `find_best_assumption` 単体（盤面の埋まり具合別）
- [x] `sudoku-solver/benches/solver.rs` を実装
  - [x] エンドツーエンド（パズル解決）
  - [x] バックトラック回数の記録 → **不要と判断**（ベンチマークで測定すべき指標ではない）
- [x] `sudoku-generator/benches/generator.rs` を実装
  - [x] エンドツーエンド（パズル生成）
- [x] `PuzzleSeed::from_str()` を実装（ベンチマークデータ読み込み用）
  - [x] TESTING.md ガイドラインに従ったテストを追加

#### 5. 測定・分析 ✅ **完了** (2026-01-23)

- [x] ベースライン測定を実行
- [x] 結果を記録（以下参照）
- [x] ベンチマーク結果を分析
- [x] ACTION-3 の要否を判断（10% 改善基準）

##### ベンチマーク結果 (2026-01-23, commit 745d274)

**`find_best_assumption` 単体** (`backtrack.rs`):

- `empty_0`: 50.2 ns
- `sparse_23`: 49.2 ns
- `mid_40`: 49.6 ns
- `dense_60`: 49.5 ns

→ 盤面の埋まり具合に関わらずほぼ一定（約50ns）。非常に高速。

**TechniqueSolver** (`solver.rs`):

- `sparse_23`: 52.8 µs
- `mid_40`: 15.1 µs
- `dense_60`: 6.8 µs
- `solution_81`: 6.1 µs

**BacktrackSolver** (`solver.rs`):

- `empty_0`: 2.93 ms (100解探索)
- `ultra_sparse_15`: 3.12 ms (100解探索)
- `sparse_23`: 52.9 µs (唯一解)
- `mid_40`: 15.2 µs (唯一解)
- `dense_60`: 6.8 µs (唯一解)
- `solution_81`: 6.2 µs (完成済み)

→ 唯一解パズルでは TechniqueSolver と BacktrackSolver の性能差はほぼなし。
→ 複数解探索（100個）で約3ms。sparse_23 (52.9µs) の約60倍程度で、妥当な性能。

**PuzzleGenerator** (`generator.rs`):

- `seed_0`: 1.91 ms
- `seed_1`: 2.09 ms
- `seed_2`: 2.15 ms

→ seedによって約10%の変動。パズル生成はソルバーの約40倍の時間。

##### 結論

- `find_best_assumption` は約50ns と非常に高速
- 盤面状態による性能差はほぼなし（最大1ns程度の誤差範囲）
- 双方向マッピング最適化で10%以上の改善は期待できない
- **ACTION-3（双方向マッピングの実装）は不要と判断**

---

## ACTION-3: 双方向マッピングの実装

- **優先度**: 中
- **依存**: ACTION-1（Pure Data Structure 化完了後）, ACTION-2（ベンチマーク結果で判断）
- **ステータス**: [✓] **完了（実施不要と判断）** (2026-01-23)
- **作業量**: 中
- **対応元**: 問題1-1

### 判断理由

ACTION-2 のベンチマーク結果により、以下の理由で実施不要と判断：

1. **`find_best_assumption` は十分高速**:
   - 実行時間は約50ns と非常に高速
   - 盤面状態による性能差はほぼなし（誤差範囲内）

2. **最適化の効果が期待できない**:
   - 双方向マッピングで10%以上の改善は困難
   - メモリ使用量の増加とコード複雑性の増加に見合わない

3. **全体のボトルネックではない**:
   - ソルバー全体の実行時間は数十µs〜数ms
   - 50ns の最適化は全体への影響が小さい

詳細な測定結果は ACTION-2 のベンチマーク結果を参照。

### 作業内容（実施しない）

1. `CandidateGrid` に `cell_candidates` フィールドを追加

   ```rust
   cell_candidates: Array81<DigitSet, PositionSemantics>
   ```

2. `place` と `remove_candidate` で同期を取る

3. `candidates_at` を O(1) に変更

4. パフォーマンステストで効果を確認

### チェックリスト

- [ ] フィールドの追加
- [ ] `place` での同期処理
- [ ] `remove_candidate` での同期処理
- [ ] `candidates_at` の実装変更
- [ ] テストの追加
- [ ] ベンチマークで効果を確認

---

## ACTION-4: ドキュメント整備とコード改善

- **優先度**: 中
- **依存**: なし
- **ステータス**: [完] 完了
- **作業量**: 中
- **対応元**: 問題1-3, 問題3-1, 懸念1, 懸念3

### 作業内容

#### コードドキュメント改善

1. **DigitGrid のドキュメント整備** ✅ **完了**
   - `Array81<Option<Digit>, PositionSemantics>` を使う理由を明記
   - `PositionSemantics` による型安全性のメリットを説明
   - 使用例の追加
   - **実施内容**: クレートレベルに「Semantics Pattern」セクションを追加し、
     すべての関連型（9ファイル、13の型/トレイト/エイリアス）からリンク
   - **コミット**: `7f7ea41` - docs(core): Add comprehensive Semantics Pattern documentation

2. **classify_cells のコメント修正** ✅ **完了**
   - bitwise DP アルゴリズムの詳細説明を追加
   - `cells[0] = FULL` から始める理由を明記
   - N個以上の候補を持つセルは追跡されないことを明記
   - **実施内容**: doc commentとインラインコメントに詳細なアルゴリズム説明を追加

#### `#[inline]` 属性の付与 ✅ **完了**

- `Array81` / `Array9` / `BitSet81` の `Index` / `IndexMut` impl
- `to_index` / `from_index` 等の変換関数（Index9/Index81のSemantics実装）
- その他の小さな関数（Position の `new`, `x`, `y`, `box_index` など）
- **実施内容**: 7ファイル（array_9.rs, array_81.rs, index_9.rs, index_81.rs, position.rs, bit_set_81.rs）に`#[inline]`を付与

#### ARCHITECTURE.md の拡充 ✅ **完了**

1. **Semantics Pattern の詳細説明**
   - Index9, Index81, Array9, Array81 の型安全性
   - PositionSemantics, DigitSemantics の役割
   - バグ防止のメリット
   - **実施内容**: lib.rsへのリンクを含む簡潔な設計判断を追加

2. **Two-grid Architecture の詳細化**
   - Digit-centric vs Cell-centric のトレードオフ
   - 各アプローチの性能特性
   - 設計判断の根拠
   - **実施内容**: 問題・解決策・メリット・トレードオフを明確化

3. **Core vs Solver の責務分離**
   - Pure Data Structure 化の意図
   - 制約ロジックの配置方針
   - 拡張性の考慮
   - **実施内容**: "mechanisms vs policies"原則を含む設計判断を追加

### チェックリスト

- [x] DigitGrid の doc comment 更新（クレートレベルに Semantics Pattern セクション追加）
- [x] classify_cells の実装コメント修正
- [x] `#[inline]` 属性の付与
- [x] ARCHITECTURE.md の存在確認と作成（必要に応じて）
- [x] ARCHITECTURE.md に Semantics Pattern の説明追加（または lib.rs からの参照）
- [x] ARCHITECTURE.md に Two-grid architecture の詳細追加
- [x] ARCHITECTURE.md に Core vs Solver の責務分離の説明追加

---

## ACTION-5: Box::leak 修正

- **優先度**: 低
- **依存**: なし
- **ステータス**: [✓]
- **作業量**: 小
- **対応元**: 問題2-2

### 作業内容

1. `PuzzleGenerator` のテストコードを調査

2. `Box::leak` を使っている箇所を特定

3. 通常のライフタイム管理に修正

### チェックリスト

- [x] テストコードの調査
- [x] `Box::leak` 使用箇所の特定
- [x] 修正実施
- [x] テストが通ることを確認

---

## ACTION-6: check_consistency API への置き換え

- **優先度**: 中
- **依存**: なし
- **ステータス**: [✓]
- **作業量**: 中
- **対応元**: 問題4-2（レビューの改善提案）

### 背景

レビューでは「`SolverError::Contradiction` が使われていない」と指摘されたが、これは事実誤認。
実際には使用されている。ただし、レビューで提案された `check_consistency() -> Result` API 自体は有用な改善である。

### 実装結果

レイヤー間の依存関係を考慮し、以下のように実装：

1. **`sudoku-core` に `ConsistencyError` を追加**
   - `derive_more` を使用してエラー型を実装
   - `SolverError` は solver レイヤーのエラー型なので core では使用できない

2. **`check_consistency() -> Result<(), ConsistencyError>` を実装**
   - `is_consistent()` の実装を基に、`Result` を返すAPIに変更

3. **`sudoku-solver` に `From<ConsistencyError>` を実装**
   - `ConsistencyError` が自動的に `SolverError::Contradiction` に変換される
   - `?` オペレータで簡潔なエラーハンドリングが可能

4. **`is_solved()` も `Result` 型に変更**
   - 矛盾検出時はエラーを返すように改善

5. **既存コードの置き換え**
   - `if !grid.is_consistent() { return Err(...) }` → `grid.check_consistency()?`
   - 冗長な `is_solved()` チェックを削除（technique solver内でチェック済み）

6. **`is_consistent()` を削除**
   - `check_consistency()` に完全に置き換え

7. **テストとドキュメントの追加**
   - `ConsistencyError`, `check_consistency()`, `is_solved()` のドキュメント
   - 各種テストケース（正常系/異常系）
   - `From<ConsistencyError>` の変換テスト

### チェックリスト

- [x] `is_consistent()` の利用箇所を調査
- [x] `ConsistencyError` を `sudoku-core` に追加
- [x] `check_consistency()` API を実装
- [x] `sudoku-solver` に `From<ConsistencyError>` を実装
- [x] `is_solved()` を `Result` 型に変更
- [x] 既存コードを `check_consistency()?` に移行
- [x] 冗長な `is_solved()` チェックを削除
- [x] `is_consistent()` を削除
- [x] `is_consistent()` の重複テスト4件を削除
- [x] ドキュメント例を `check_consistency()` に更新
- [x] テストの追加（`check_consistency`, `is_solved`, `From` 変換）
- [x] ドキュメントコメントの追加
- [x] `cargo test --all` で全テスト通過確認
- [x] `cargo clippy --all-targets` で警告なし確認

---

## ACTION-7: BacktrackSolver のテスト調査

- **優先度**: 低
- **依存**: なし
- **ステータス**: [✓]
- **作業量**: 小
- **対応元**: 問題5-2

### 作業内容

1. 現在のテストカバレッジを確認

2. 複数解を持つパズルのテストが存在するか調査

3. 不足していれば、テストを追加

### 調査結果

既存の `test_multiple_solutions` テストが存在していたが、以下の点で不十分：

- 解が異なることを検証していない
- バックトラックが実際に発生しているかを検証していない
- 統計情報（backtrack_count など）の検証が不足

### 追加したテスト

1. **`test_multiple_solutions`** を拡張
   - 2つの解が実際に異なることを検証

2. **`test_multiple_solutions_with_partial_grid`**
   - 部分的に埋まったグリッドから複数解を生成
   - 全ての解が有効で異なることを検証
   - 元の配置が保持されることを確認

3. **`test_backtracking_occurs`**
   - バックトラックが必要な状況でassumptionsが記録されることを検証

4. **`test_backtrack_count_increments`**
   - 複数解の探索中にバックトラックカウントが追跡されることを確認

5. **`test_solution_is_complete`**
   - 解が完全（全81セル）であることを検証

### チェックリスト

- [x] テストコードの調査
- [x] カバレッジの確認
- [x] 必要に応じてテスト追加
- [x] 全テストが通ることを確認

---

## 推奨作業順序

すべてのアクションが完了しました。

### 完了済み

- ✅ ACTION-1: Pure Data Structure 化（2026-01-23）
- ✅ ACTION-2: ベンチマークの追加（2026-01-23）
- ✅ ACTION-3: 双方向マッピングの実装（2026-01-23、実施不要と判断）
- ✅ ACTION-4: ドキュメント整備とコード改善（2026-01-22）
- ✅ ACTION-5: Box::leak 修正（2026-01-22）
- ✅ ACTION-6: check_consistency API への置き換え（2026-01-22）
- ✅ ACTION-7: BacktrackSolver のテスト調査（2026-01-22）

---

## 対応履歴

**注**: 新しいものほど下に記載されています（時系列順）。

### 2026-01-22

- ACTION-5 完了（Box::leak 修正）
  - `crates/sudoku-generator/src/lib.rs` のテストコードから `Box::leak` を削除
  - `create_test_generator()` ヘルパー関数を削除し、各テスト関数内で `TechniqueSolver` を直接作成
  - 通常のライフタイム管理に修正
  - コミット: [e8ef0d5](https://github.com/gifnksm/sudoku/commit/e8ef0d5) - test(generator): remove Box::leak from test helper

- ACTION-6 完了（check_consistency API への置き換え）
  - `sudoku-core` に `ConsistencyError` を追加（`derive_more` 使用）
  - `CandidateGrid::check_consistency() -> Result<(), ConsistencyError>` を実装
  - `CandidateGrid::is_solved()` を `Result<bool, ConsistencyError>` に変更
  - `sudoku-solver` に `From<ConsistencyError> for SolverError` を実装
  - `is_consistent()` の呼び出しを `check_consistency()?` に置き換え
  - テストとドキュメントを追加
  - コミット: [b1e563c](https://github.com/gifnksm/sudoku/commit/b1e563c) - refactor(core,solver): replace is_consistent with check_consistency API
  - **注記**: `is_consistent()` メソッド本体の削除が未完了だった（後日対応）

- ACTION-7 完了（BacktrackSolver のテスト調査）
  - `BacktrackSolver` のテストカバレッジを調査
  - 既存の `test_multiple_solutions` を拡張し、解の差異を検証
  - 5つの新しいテストケースを追加
  - バックトラックの正当性とstatistics収集が適切に動作することを確認
  - 全テスト（14個）が通過することを確認
  - コミット: [6b8f87a](https://github.com/gifnksm/sudoku/commit/6b8f87a) - Complete ACTION-7: Add comprehensive tests for BacktrackSolver

- ACTION-4 部分完了（1-(a) DigitGrid のドキュメント整備）
  - クレートレベルに「Semantics Pattern: Type-Safe Indexing」セクションを追加
  - すべての関連型（9ファイル、13の型/トレイト/エイリアス）からリンク
  - 3つの主要な目的を明確化：型安全性、実装の共通化、効率的なデータ構造
  - コミット: [7f7ea41](https://github.com/gifnksm/sudoku/commit/7f7ea41) - docs(core): Add comprehensive Semantics Pattern documentation

- ACTION-4 完了（ドキュメント整備とコード改善）
  - classify_cells のコメント修正（bitwise DP アルゴリズム説明）
  - `#[inline]` 属性の付与（7ファイル）
  - ARCHITECTURE.md の拡充（Semantics Pattern, Two-grid, Core vs Solver）
  - コミット: [30164eb](https://github.com/gifnksm/sudoku/commit/30164eb) - feat(review): Complete ACTION-4

- ACTION-6 追加対応（`is_consistent()` の削除漏れ対応）
  - 残っていた `is_consistent()` メソッドとそのdocコメントを削除
  - 重複テスト4件を削除（`test_is_consistent_*`）
  - `lib.rs` のドキュメント例を `check_consistency().is_ok()` に更新
  - コミット: [69ca2b4](https://github.com/gifnksm/sudoku/commit/69ca2b4) - refactor(core): remove deprecated is_consistent() method

- ACTION-1 完了（Pure Data Structure 化）
  - `CandidateGrid::place` から制約伝播を削除（配置セルの候補のみを更新）
  - `NakedSingle::apply` に制約伝播を追加（row/column/box からの候補除外）
  - `place_no_propagation`, `from_digit_grid_no_propagation`, `from_str_no_propagation` を削除
  - テストを更新（propagation 関連のテストを削除、decided_cells テストを追加）
  - **最終確認作業で追加対応**:
    - `BacktrackSolver::pure_backtrack()` を `without_techniques()` にリネーム
      - 理由: "pure" という用語が ACTION-1 の "Pure Data Structure" と混同されるため
      - "pure backtracking" → "backtracking without techniques" に変更
    - `docs/ARCHITECTURE.md` を更新（3箇所の constraint propagation 関連記述を修正）
  - コミット: [eab41ed](https://github.com/gifnksm/sudoku/commit/eab41ed)

- ACTION-2 の内容を更新
  - レビュー指摘の内容を再確認（`candidates_at`, `classify_cells`, `decided_cells` の性能問題）
  - `candidates_at` の呼び出し箇所を調査（実際には `find_best_assumption` のみで使用）
  - `CandidateGrid` の全公開メソッドを調査し、cell-oriented で高速化される読み取り専用メソッドは `candidates_at` のみと確認
  - `to_digit_grid` の最適化経緯（コミット [5cdd9302](https://github.com/gifnksm/sudoku/commit/5cdd9302)）を背景情報として追加
  - 検討事項セクションを追加：ベンチマークフレームワーク選定、テストデータ準備方法など
  - ベンチマーク対象を整理：`candidates_at` 単体、`find_best_assumption`、エンドツーエンド
  - 一般的なベンチマークの追加を提案（今後の最適化に有用）
  - 判断基準を明確化：10% 以上の改善が見込めれば ACTION-3 を実施
  - 前提条件として `find_best_assumption` の共通化を追加（方法は要検討）
  - チェックリストをグループ化（リファクタリング、環境セットアップ、ベンチマーク実装、測定・分析）
  - コミット: [3385964](https://github.com/gifnksm/sudoku/commit/3385964) - docs(review): Update ACTION-2 with detailed investigation results

### 2026-01-23

- ACTION-2 検討事項決定
  - **ベンチマークフレームワーク**: Criterion.rs を採用
    - 理由: 統計的分析、HTMLレポート、回帰検出、stable Rust 対応、CI 対応可能
  - **`find_best_assumption` の共通化方法**: `sudoku-solver/src/backtrack.rs` に新規モジュールを作成
    - `pub mod backtrack` として公開（re-export なし）
    - 関数名は `find_best_assumption` のまま維持
    - 使用例: `sudoku_solver::backtrack::find_best_assumption(&grid)`
  - **ベンチマーク対象の選定原則**: 最小限（ACTION-3 判断用）に集中
    - 測定する価値がある操作: 複雑な計算、頻繁な呼び出し、最適化の余地、重要な性能
    - 測定不要: 単純なメモリ確保、トリビアルなゲッター、コンパイラ最適化可能な操作
    - `candidates_at` 単体はスキップ（`find_best_assumption` で間接測定）
  - **ベンチマークのクレート配置**: 各クレートの `benches/` ディレクトリ
    - Rust の標準的なプラクティスに従う
  - **テストデータ**: `PuzzleGenerator` で生成（seed 固定）
    - 現在のソルバー（NakedSingle + HiddenSingle のみ）では難問が解けないため
    - 再現性確保のため seed を記録
  - **テクニックセットの固定**: `fundamental_techniques()` 関数を追加
    - 将来テクニックが追加されても、ベンチマーク結果の意味が変わらないようにする
    - `TechniqueSolver::with_fundamental_techniques()` と `BacktrackSolver::with_fundamental_techniques()` を追加
  - コミット: [442113d](https://github.com/gifnksm/sudoku/commit/442113d) - docs: Complete ACTION-2 design decisions

- ACTION-2 タスク1完了（リファクタリング）
  - **新規モジュール**: `sudoku-solver/src/backtrack.rs` (214行)
    - `find_best_assumption` 関数を実装
    - MRV (Minimum Remaining Values) ヒューリスティック
    - 包括的なモジュール・関数ドキュメント
    - 5つのテストケース（正常系2、パニック系2、検証系1）
  - **コード削減**: 39行（generator: 34行、backtrack_solver: 5行）
  - **公開API**: `pub mod backtrack` として `sudoku-solver` から公開
  - **ドキュメント更新**: `generator` のドキュメントに `backtrack` モジュールの使用を明記
  - **既存コード改善**: `DigitGrid::fmt` の `Position::ALL` イテレーションパターンを修正
  - コミット: [4efefd8](https://github.com/gifnksm/sudoku/commit/4efefd8) - refactor: extract find_best_assumption to backtrack module (ACTION-2)

- ACTION-2 タスク2完了（テクニックセットの固定）
  - **新規関数**: `technique::fundamental_techniques()` (Naked Single, Hidden Single)
    - TechniqueSolver の基礎となる最も基本的なテクニック
    - ベンチマークの安定したベースラインとして機能
    - 包括的なドキュメントコメント（TechniqueSolver のコンテキストで説明）
  - **`all_techniques()` の更新**: `fundamental_techniques()` をベースとする構造に変更（将来の拡張性を考慮）
  - **新規コンストラクタ**:
    - `TechniqueSolver::with_fundamental_techniques()`: fundamental techniques を使用する TechniqueSolver
    - `BacktrackSolver::with_fundamental_techniques()`: fundamental techniques を使用する BacktrackSolver
    - ベンチマーク用の標準構成として提供
  - **テスト**: 全テスト通過（ユニットテスト + ドキュメントテスト）、警告なし
  - コミット: [ef1eec0](https://github.com/gifnksm/sudoku/commit/ef1eec0) - feat: add fundamental_techniques() and with_fundamental_techniques() (ACTION-2)

- ACTION-2 タスク3/4完了（ベンチマーク環境のセットアップとベンチマーク実装）
  - **Criterion.rs の追加**: `workspace.dependencies` と各クレートの `dev-dependencies` に追加
  - **`PuzzleSeed::from_str()` の実装**: ベンチマークでseed文字列からパズルを生成するために実装
    - 64文字の16進数文字列をパース
    - TESTING.md ガイドラインに従った3つの統合テスト（有効ケース、ラウンドトリップ、エラーケース）
  - **ベンチマーク実装**:
    - `sudoku-solver/benches/backtrack.rs`: `find_best_assumption` 単体（empty/sparse/mid/dense の4パターン）
    - `sudoku-solver/benches/solver.rs`: TechniqueSolver と BacktrackSolver のエンドツーエンド
    - `sudoku-generator/benches/generator.rs`: PuzzleGenerator のエンドツーエンド（3つの固定seed）
  - **テストデータ**: 固定seedから生成した複数の難易度のパズルを使用
  - 各ベンチマークファイルに包括的なドキュメントコメントを追加
  - コミット: [745d274](https://github.com/gifnksm/sudoku/commit/745d274) - feat: add benchmarks for solver and generator

- ACTION-2 タスク5完了（測定・分析）とACTION-3決定
  - **ベンチマーク結果**:
    - `find_best_assumption`: 約50ns（盤面状態による差はほぼなし）
    - TechniqueSolver: sparse(52.8µs) → dense(6.8µs)
    - BacktrackSolver: 唯一解パズルはTechniqueSolverとほぼ同等、複数解探索(100個)で約3ms
    - PuzzleGenerator: 約2ms（seedにより1.9-2.2ms）
  - **分析結果**:
    - `find_best_assumption` は非常に高速で、双方向マッピング最適化の効果は期待できない
    - 盤面状態による性能差はほぼなし（誤差範囲内）
    - 全体のボトルネックではない（ソルバー全体は数十µs〜数ms）
  - **ACTION-3 の判断**: **実施不要**
    - 理由: 10%以上の改善は期待できず、メモリ増加とコード複雑化に見合わない
    - ACTION-3 のステータスを [不要] に更新
  - action.md に詳細な測定結果と分析を記録
  - コミット: [7f7a182](https://github.com/gifnksm/sudoku/commit/7f7a182) - docs(review): Complete ACTION-2 and decide ACTION-3 is not needed
