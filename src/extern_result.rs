// Copyright 2018 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

extern crate mentat_ffi;

use std; 
use std::os::raw::c_char;

use mentat_ffi::utils::strings::{
    string_to_c_char,
};

// TODO unify this with mentat_ffi's ExternResult: https://github.com/mozilla/mentat/issues/710
#[repr(C)]
#[derive(Debug)]
pub struct ExternResult {
    pub ok: *const c_char,
    pub err: *const c_char,
}

impl<T, E> From<Result<T, E>> for ExternResult where E: std::error::Error, T: std::string::ToString {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(value) => {
                ExternResult {
                    err: std::ptr::null(),
                    ok: string_to_c_char(value.to_string()),
                }
            },
            Err(e) => {
                ExternResult {
                    err: string_to_c_char(e.to_string()),
                    ok: std::ptr::null(),
                }
            }
        }
    }
}
