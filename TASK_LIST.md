# Deterministic Task List for Shimmy Console Shakedown

## Phase 1: Discovery and Setup
- [x] Read all Copilot instructions (.github/copilot-instructions.md and .github/copilot-instructions-shimmy-console.md)
- [x] Update Copilot instructions with Shimmy Vision workflow and theme validation steps
- [x] Set up tasks.json with shimmy-dev-default task to avoid stomping bash processes

## Phase 2: System Shakedown
- [ ] Validate shimmy-default theme loading and basic functionality
- [ ] Test chat session with local AI model (speed and responsiveness)
- [ ] Verify all 16 tools are accessible and functional
- [ ] Check tool manifests and plugin system (especially read_image as plugin example)
- [ ] Ensure metrics streaming works (tokens, system info) via WebSocket
- [ ] Test metrics panel in shimmy-default theme with checkboxes and dropdown

## Phase 3: Tool and Plugin Validation
- [ ] Conduct Q&A session with local AI on tool usage
- [ ] Verify AI can see and understand all tools
- [ ] Test plugin extensibility (add/remove tools via manifests)
- [ ] Fix any issues with tool registration or execution

## Phase 4: Theme Validation and Iteration
- [ ] Use Shimmy Vision for visual validation of themes (screenshots)
- [ ] Validate theme generation from schema
- [ ] Ensure performance and correctness in theme rendering
- [ ] Iterate on metrics panel for full feature set

## Phase 5: Architecture and Documentation
- [ ] Document all layers and architecture patterns
- [ ] Ensure ease of use for AI assistants and users
- [ ] Final validation of plugin ecosystem

## Notes
- Shimmy Vision: Separate executable for vision tasks, integrate via read_image tool example.
- Metrics: Ensure breakaway panel with selective display via checkboxes/dropdown.
- Tools: 15 built-in, 1 plugin (read_image) to demonstrate extensibility.
- Run shimmy dev via tasks.json task to prevent bash stomping.