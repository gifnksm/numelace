# Testing Guidelines

## Purpose

This document provides guidelines for writing effective tests in this project, with a focus on avoiding over-testing and maintaining high-quality test coverage. It captures lessons learned from actual test reduction efforts and Rust-specific considerations.

## Unit Test Principles

### Scope and Responsibilities

**Unit tests focus on a single unit** (function, type, module) in isolation:

- Test the unit's behavior (public API and complex private logic)
- Minimize external dependencies
- Execute quickly, independently, and deterministically

### What to Test

Focus on these aspects:

- **Various inputs** - Normal cases, boundary values, and edge cases
- **Error conditions** - Invalid inputs and error handling
- **Expected panics** - Use `#[should_panic]` (keep these tests separate)

### What NOT to Test (Rust-Specific Over-Testing)

These patterns add maintenance cost without sufficient value:

1. **Compiler-generated code** - Derived traits

   ```rust
   // ❌ Don't test: Trust the compiler's derive implementation
   #[derive(Clone, Debug, PartialEq, Eq)]
   struct MyType { ... }
   
   #[test]
   fn test_clone() { ... }  // Unnecessary
   ```

2. **Obvious implementations** - Dedicated tests for self-evident properties

   ```rust
   // ❌ Don't create dedicated test functions for obvious properties
   #[test]
   fn test_union_is_commutative() {
       assert_eq!(a | b, b | a);
   }
   
   // ✅ Do verify key properties inline within table-driven tests
   #[test]
   fn test_union() {
       let cases = [(set![0, 1], set![1, 2], set![0, 1, 2])];
       for (a, b, expected) in cases {
           assert_eq!(a.union(&b), expected);
           assert_eq!(b.union(&a), expected);  // Commutativity (1 line)
       }
   }
   ```

   **Rationale**: While properties may seem obvious, implementation bugs (e.g., using `&` instead of `|`) can occur. Verifying key properties inline adds minimal cost (1-2 lines) while catching real bugs and enabling safe refactoring.

3. **Self-evident invariants** - Obvious relationships

   ```rust
   // ❌ Don't test: If len() is broken, other tests will fail
   #[test]
   fn test_len_equals_iter_count() {
       assert_eq!(set.len(), set.iter().count());
   }
   ```

4. **Trivial delegation** - Delegation without consistency verification

   ```rust
   // ❌ Don't test delegation implementation itself
   impl BitSet {
       pub fn union(&self, other: &Self) -> Self {
           self | other  // Just delegates to operator
       }
   }
   
   // ✅ Do verify method-operator consistency in existing tests
   #[test]
   fn test_union() {
       let cases = [(set![0, 1], set![1, 2], set![0, 1, 2])];
       for (a, b, expected) in cases {
           assert_eq!(a.union(&b), expected);
           assert_eq!(a | b, expected);  // Consistency (1 line)
       }
   }
   ```

   **Rationale**: While delegation itself is trivial, consistency between methods and operators is part of the public API contract. This catches refactoring errors where one implementation changes independently.

5. **Exhaustive enumeration** - Testing every individual case

   ```rust
   // ❌ Don't test each digit individually
   #[test]
   fn test_digit_1() { assert_eq!(Digit::D1.value(), 1); }
   #[test]
   fn test_digit_2() { assert_eq!(Digit::D2.value(), 2); }
   // ... 7 more tests
   
   // ✅ Do test boundaries + loop for completeness
   #[test]
   fn test_digit_values() {
       assert_eq!(Digit::D1.value(), 1);  // Min
       assert_eq!(Digit::D9.value(), 9);  // Max
       
       // All digits round-trip
       for digit in Digit::ALL {
           assert_eq!(Digit::from_value(digit.value()), digit);
       }
   }
   ```

### What to Test Carefully (Easy to Miss)

1. **Manual trait implementations** - Especially for types with `PhantomData`

   ```rust
   // ✅ Do test: Manual implementation needs verification
   struct Array81<T, S> {
       data: [T; 81],
       _phantom: PhantomData<S>,
   }
   
   // Manual Clone implementation must handle PhantomData correctly
   impl<T: Clone, S> Clone for Array81<T, S> { ... }
   
   #[test]
   fn test_clone_with_phantom_data() {
       let arr1 = Array81::from([1; 81]);
       let arr2 = arr1.clone();
       assert_eq!(arr1, arr2);
   }
   ```

2. **Code not executed at runtime** - Const functions, macro-generated code
   - Simple constant definitions (`const MAX: usize = 100;`) don't need tests
   - Complex compile-time computations should be tested

   ```rust
   // ✅ Do test: Complex compile-time computation needs verification
   const ROW_POSITIONS: [DigitPositions; 9] = compute_row_positions();
   
   #[test]
   fn test_row_positions() {
       // Test properties, not individual values
       for y in 0..9 {
           assert_eq!(ROW_POSITIONS[y].len(), 9);
       }
       
       // Rows are disjoint
       for y1 in 0..9 {
           for y2 in (y1 + 1)..9 {
               assert_eq!((ROW_POSITIONS[y1] & ROW_POSITIONS[y2]).len(), 0);
           }
       }
   }
   ```

### Effective Techniques

1. **Table-driven tests** - Multiple cases with same logic

   ```rust
   #[test]
   fn test_union() {
       let cases = [
           (set![0, 1], set![1, 2], set![0, 1, 2]),
           (set![], set![0, 1], set![0, 1]),  // Empty is identity
           (set![0, 1], set![], set![0, 1]),
       ];
       for (a, b, expected) in cases {
           assert_eq!(a.union(&b), expected);
           assert_eq!(b.union(&a), expected);  // Commutativity
           assert_eq!(a | b, expected);        // Operator consistency
       }
   }
   ```

   **Tip**: Inline property checks (commutativity, consistency) add 1-2 lines but catch implementation bugs and enable safe refactoring with minimal maintenance cost.

2. **Loop-based validation** - Test all cases without enumeration

   ```rust
   // Test boundary values explicitly + loop for completeness
   for value in 0..9 {
       let index = Semantics::to_index(value);
       let result = Semantics::from_index(index);
       assert_eq!(result, value);
   }
   ```

3. **Property-based comments** - Describe what, not how

   ```rust
   // ✅ Good: States the property being verified
   // Insert is idempotent
   assert!(!set.insert(0));
   
   // ❌ Bad: Just describes the code
   // Insert 0 and check it returns false
   ```

4. **Consolidated tests** - Group related assertions

   ```rust
   #[test]
   fn test_basic_operations() {
       // Multiple related checks in one test
       // Insert is idempotent
       assert!(set.insert(0));
       assert!(!set.insert(0));
       
       // Remove is idempotent
       assert!(set.remove(0));
       assert!(!set.remove(0));
   }
   ```

5. **Keep `should_panic` tests isolated** - Cannot mix with other assertions

   ```rust
   // Each should_panic test must be separate
   #[test]
   #[should_panic(expected = "out of bounds")]
   fn test_rejects_invalid() {
       Index9::new(10);  // Only one assertion allowed
   }
   ```

## Integration Test Principles

Integration tests verify that components work correctly together:

- Test complete user scenarios and workflows
- Verify component interactions
- Cover end-to-end flows

**Integration tests should be evaluated carefully** - each test should cover a distinct scenario. Redundant integration tests can and should be reduced, but this is less common than redundant unit tests.

## Project Testing Philosophy

1. **Trust the compiler for derived traits** - Don't test what Rust guarantees (e.g., derived `Clone`, `Debug`), but do test manual implementations
2. **Test human-written logic** - Focus on manual implementations and complex computations
3. **Optimize for maintenance** - Tests should be easy to understand and update
4. **Enable confident refactoring** - Tests should catch real regressions, not implementation details
5. **Balance coverage and cost** - Every test has a maintenance cost; ensure it provides value
