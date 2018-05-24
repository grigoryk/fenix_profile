// Copyright 2018 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

extern crate mentat;
extern crate time;

use mentat::TypedValue;
use time::Timespec;

pub trait ToTypedValue {
    fn to_typed_value(&self) -> TypedValue;
}

impl ToTypedValue for Timespec {
    fn to_typed_value(&self) -> TypedValue {
        let micro_seconds = (self.sec * 1_000_000) + i64::from(self.nsec / 1_000);
        TypedValue::instant(micro_seconds)
    }
}