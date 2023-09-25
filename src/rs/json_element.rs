/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use pyo3::prelude::*;

#[derive(Debug, Clone, FromPyObject)]
pub enum JsonElement<'a> {
    #[pyo3(transparent, annotation = "int")]
    Int(u64),
    #[pyo3(transparent, annotation = "str")]
    String(String),
    //None,
    #[pyo3(transparent)]
    CatchAll(&'a PyAny), // To make PyO3 stop from complaining
}
