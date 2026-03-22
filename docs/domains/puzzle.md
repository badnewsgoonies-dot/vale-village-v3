# Domain: puzzle
## Scope: src/domains/puzzle/
## Imports from contract
PuzzleDef, PuzzleType, Element, DjinnId, DialogueSideEffect

## Deliverables
1. PuzzleState enum: Unsolved, InProgress, Solved
2. PuzzleInstance struct: def reference, state, progress data
3. fn can_attempt(puzzle: &PuzzleDef, context: &dyn PuzzleContext) -> bool
4. fn attempt_solve(instance: &mut PuzzleInstance, input: PuzzleInput) -> PuzzleResult
5. PuzzleInput enum: PushDirection(Direction), ActivateElement(Element), UseDjinn(DjinnId), ToggleSwitch(usize), SlideDirection(Direction)
6. PuzzleResult enum: Solved(Option<DialogueSideEffect>), Progress, Failed, AlreadySolved
7. PuzzleContext trait: has_djinn, has_element_ability
8. Tests: each puzzle type solve path, already-solved rejection, reward delivery

## Does NOT handle
Rendering puzzles, animation, player input mapping.

## Quantitative targets
- 5 PuzzleType variants: PushBlock, ElementPillar, DjinnPuzzle, SwitchSequence, IceSlide
- ≥6 tests
