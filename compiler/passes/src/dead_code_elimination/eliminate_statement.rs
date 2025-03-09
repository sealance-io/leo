// Copyright (C) 2019-2025 Provable Inc.
// This file is part of the Leo library.

// The Leo library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Leo library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Leo library. If not, see <https://www.gnu.org/licenses/>.

use crate::DeadCodeEliminator;

use leo_ast::{
    AssignStatement,
    Block,
    ConsoleStatement,
    DefinitionStatement,
    Expression,
    ExpressionReconstructor,
    ExpressionStatement,
    IterationStatement,
    Statement,
    StatementReconstructor,
};

impl StatementReconstructor for DeadCodeEliminator<'_> {
    /// Reconstruct an assignment statement by eliminating any dead code.
    fn reconstruct_assign(&mut self, mut input: AssignStatement) -> (Statement, Self::AdditionalOutput) {
        // Check the lhs of the assignment to see any of variables are used.
        let lhs_is_used = match &input.place {
            Expression::Identifier(identifier) => self.used_variables.contains(&identifier.name),
            Expression::Tuple(tuple_expression) => tuple_expression.elements.iter().any(|element| match element {
                Expression::Identifier(identifier) => self.used_variables.contains(&identifier.name),
                _ => panic!("The previous compiler passes guarantee the tuple elements on the lhs are identifiers."),
            }),
            _ => panic!(
                "The previous compiler passes guarantee that `place` is either an identifier or tuple of identifiers."
            ),
        };

        if !lhs_is_used && self.side_effect_free(&input.value) {
            // We can eliminate this statement.
            (Statement::dummy(), Default::default())
        } else {
            // We still need it.
            input.value = self.reconstruct_expression(input.value).0;
            (Statement::Assign(Box::new(input)), Default::default())
        }
    }

    /// Reconstructs the statements inside a basic block, eliminating any dead code.
    fn reconstruct_block(&mut self, block: Block) -> (Block, Self::AdditionalOutput) {
        // Reconstruct each of the statements in reverse.
        let mut statements: Vec<Statement> =
            block.statements.into_iter().rev().map(|statement| self.reconstruct_statement(statement).0).collect();

        statements.retain(|stmt| !stmt.is_empty());

        // Reverse the direction of `statements`.
        statements.reverse();

        (Block { statements, span: block.span, id: block.id }, Default::default())
    }

    /// Parsing guarantees that console statements are not present in the program.
    fn reconstruct_console(&mut self, _: ConsoleStatement) -> (Statement, Self::AdditionalOutput) {
        panic!("`ConsoleStatement`s should not be in the AST at this phase of compilation.")
    }

    /// Static single assignment replaces definition statements with assignment statements.
    fn reconstruct_definition(&mut self, _: DefinitionStatement) -> (Statement, Self::AdditionalOutput) {
        panic!("`DefinitionStatement`s should not exist in the AST at this phase of compilation.")
    }

    /// Loop unrolling unrolls and removes iteration statements from the program.
    fn reconstruct_iteration(&mut self, _: IterationStatement) -> (Statement, Self::AdditionalOutput) {
        panic!("`IterationStatement`s should not be in the AST at this phase of compilation.");
    }

    fn reconstruct_expression_statement(
        &mut self,
        mut input: ExpressionStatement,
    ) -> (Statement, Self::AdditionalOutput) {
        if self.side_effect_free(&input.expression) {
            (Statement::dummy(), Default::default())
        } else {
            input.expression = self.reconstruct_expression(input.expression).0;
            (Statement::Expression(input), Default::default())
        }
    }
}
