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

| ID       | 優先度 | 依存               | ステータス | 概要                                   | 対応元                              |
| -------- | ------ | ------------------ | ---------- | -------------------------------------- | ----------------------------------- |
| ACTION-1 | 高     | -                  | [ ]        | Pure Data Structure 化（テスト追加含） | 問題2-1, 問題3-2, 問題5-1, 懸念2    |
| ACTION-2 | 高     | -                  | [ ]        | ベンチマークの追加                     | 問題1-1                             |
| ACTION-3 | 中     | ACTION-1, ACTION-2 | [ ]        | 双方向マッピングの実装                 | 問題1-1                             |
| ACTION-4 | 中     | -                  | [ ]        | ドキュメント整備とコード改善           | 問題1-3, 問題3-1, 懸念1, 懸念3      |
| ACTION-5 | 低     | -                  | [✓]        | Box::leak 修正                         | 問題2-2                             |
| ACTION-6 | 中     | -                  | [ ]        | check_consistency API への置き換え     | 問題4-2                             |
| ACTION-7 | 低     | -                  | [ ]        | BacktrackSolver のテスト調査           | 問題5-2                             |

---

## ACTION-1: Pure Data Structure 化

- **優先度**: 高（ブロッカー）
- **依存**: なし
- **ステータス**: [ ]
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

- [ ] `CandidateGrid::place` の実装変更（制約伝播の削除）
- [ ] `NakedSingle::apply` の実装変更（制約伝播の追加）
- [ ] `place_no_propagation` の削除
- [ ] `CandidateGrid` のテスト更新
- [ ] `NakedSingle` のテスト更新（制約伝播の検証）
- [ ] 既存の統合テストが通ることを確認
- [ ] ドキュメントの更新（Pure Data Structure の説明）

---

## ACTION-2: ベンチマークの追加

- **優先度**: 高
- **依存**: なし
- **ステータス**: [ ]
- **作業量**: 中
- **対応元**: 問題1-1

### 作業内容

1. ベンチマークファイルの作成
    - `crates/sudoku-core/benches/candidate_grid.rs`
    - `crates/sudoku-generator/benches/puzzle_generation.rs`

2. ベンチマーク対象
    - `CandidateGrid::candidates_at` の呼び出しコスト
    - `CandidateGrid::classify_cells` の実行時間
    - パズル生成全体の時間

### 判断基準

ベンチマーク結果を見て、ACTION-3 (双方向マッピング) の要否を判断：

- **判断観点**: レビューで指摘された `candidates_at` の呼び出しコストがボトルネックになっているか
- **測定項目**:
  - `candidates_at` 単体の実行時間
  - `classify_cells` の実行時間（内部で `candidates_at` を多用）
  - パズル生成全体の時間（バックトラック探索での影響）
- **判断基準**: パズル生成時間への影響が有意であれば、双方向マッピングを実装

### チェックリスト

- [ ] `candidates_at` のベンチマーク追加
- [ ] `classify_cells` のベンチマーク追加
- [ ] パズル生成のベンチマーク追加
- [ ] ベースラインの記録
- [ ] ベンチマーク結果を分析し、ACTION-3 の要否を判断

---

## ACTION-3: 双方向マッピングの実装

- **優先度**: 中
- **依存**: ACTION-1（Pure Data Structure 化完了後）, ACTION-2（ベンチマーク結果で判断）
- **ステータス**: [ ]
- **作業量**: 中
- **対応元**: 問題1-1

### 前提条件

- ACTION-2 のベンチマーク結果で、双方向マッピングが必要と判断された場合のみ実施

### 作業内容

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
- **ステータス**: [ ]
- **作業量**: 中
- **対応元**: 問題1-3, 問題3-1, 懸念1, 懸念3

### 作業内容

#### コードドキュメント改善

1. **DigitGrid のドキュメント整備**
    - `Array81<Option<Digit>, PositionSemantics>` を使う理由を明記
    - `PositionSemantics` による型安全性のメリットを説明
    - 使用例の追加

2. **classify_cells のコメント修正**
    - bitwise DP アルゴリズムの詳細説明を追加
    - `cells[0] = FULL` から始める理由を明記
    - N個以上の候補を持つセルは追跡されないことを明記

#### `#[inline]` 属性の付与

- `Array81` / `Array9` / `BitSet` の `Index` / `IndexMut` impl
- `to_index` / `from_index` 等の変換関数
- その他の小さな関数（Position の `box_index` など）

#### ARCHITECTURE.md の拡充

1. **Semantics Pattern の詳細説明**
    - Index9, Index81, Array9, Array81 の型安全性
    - PositionSemantics, DigitSemantics の役割
    - バグ防止のメリット

2. **Two-grid Architecture の詳細化**
    - Digit-centric vs Cell-centric のトレードオフ
    - 各アプローチの性能特性
    - 設計判断の根拠

3. **Core vs Solver の責務分離**
    - Pure Data Structure 化の意図
    - 制約ロジックの配置方針
    - 拡張性の考慮

### チェックリスト

- [ ] DigitGrid の doc comment 更新
- [ ] classify_cells の実装コメント修正
- [ ] `#[inline]` 属性の付与
- [ ] ARCHITECTURE.md に Semantics Pattern の説明追加
- [ ] ARCHITECTURE.md に Two-grid architecture の詳細追加
- [ ] ARCHITECTURE.md に Core vs Solver の責務分離の説明追加

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
- **ステータス**: [ ]
- **作業量**: 中
- **対応元**: 問題4-2（レビューの改善提案）

### 背景

レビューでは「`SolverError::Contradiction` が使われていない」と指摘されたが、これは事実誤認。
実際には使用されている。ただし、レビューで提案された `check_consistency() -> Result` API 自体は有用な改善である。

### 作業内容

1. `is_consistent() -> bool` の利用箇所を調査

2. `check_consistency() -> Result<(), SolverError>` API を追加

3. `is_consistent()` の呼び出しを `check_consistency()` に置き換え

4. `is_consistent()` を削除

5. `?` オペレータでエラーハンドリングを簡潔化

### チェックリスト

- [ ] `is_consistent()` の利用箇所を調査
- [ ] `check_consistency()` API を実装
- [ ] 既存コードを `check_consistency()` に移行
- [ ] `is_consistent()` を削除
- [ ] テストの更新

---

## ACTION-7: BacktrackSolver のテスト調査

- **優先度**: 低
- **依存**: なし
- **ステータス**: [ ]
- **作業量**: 小
- **対応元**: 問題5-2

### 作業内容

1. 現在のテストカバレッジを確認

2. 複数解を持つパズルのテストが存在するか調査

3. 不足していれば、テストを追加

### チェックリスト

- [ ] テストコードの調査
- [ ] カバレッジの確認
- [ ] 必要に応じてテスト追加

---

## 推奨作業順序

### Phase 1: すぐできる改善（並行作業可能）

以下は互いに依存せず、どれから着手しても構いません：

- ACTION-4: ドキュメント整備とコード改善
- ACTION-5: Box::leak 修正
- ACTION-6: check_consistency API への置き換え
- ACTION-7: BacktrackSolver のテスト調査
- ACTION-2: ベンチマークの追加（ACTION-3 の判断材料）

### Phase 2: Pure Data Structure 化（ブロッカー）

- **ACTION-1**: Core を Pure Data Structure 化
  - 問題3-2（NakedSingle の実装変更）を含む
  - 問題5-1（テストコードの修正）を含む
  - 完了後、ACTION-3 の実装が容易になる

### Phase 3: ベンチマーク結果に基づく判断

- ACTION-2 のベンチマーク結果を評価
- 必要であれば **ACTION-3** (双方向マッピング) を実装

---

## 対応履歴

<!-- アクションが完了したら、ここに記録 -->

- **2026-01-21**: ACTION-5 完了
  - `crates/sudoku-generator/src/lib.rs` のテストコードから `Box::leak` を削除
  - `create_test_generator()` ヘルパー関数を削除し、各テスト関数内で `TechniqueSolver` を直接作成
  - 通常のライフタイム管理に修正
  - lint、コンパイル、テスト通過確認済み
