// Copyright 2018 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

#[macro_use(bail)]
extern crate failure;
extern crate time;

#[macro_use(kw, var)]
extern crate mentat;

extern crate mentat_ffi;

extern crate libc;

use failure::{
    Error,
    err_msg,
};

use time::Timespec;

use libc::{
    time_t,
};

use std::os::raw::{
    c_char,
};

use std::sync::Arc;

use mentat::{
    Store,
    QueryInputs,
    ValueType,
    Queryable,
    TypedValue,
    IntoResult,
    HasSchema,
};

use mentat::entity_builder::{
    TermBuilder,
};

use mentat::vocabulary::{
    AttributeBuilder,
    Definition,
    VersionedStore,
};

use mentat::vocabulary::attribute::{
    Unique
};

use mentat_ffi::{
    BuildTerms,
};

use mentat_ffi::utils::strings::{
    c_char_to_string,
    string_to_c_char,
};

mod extern_result;
use extern_result::ExternResult;

mod utils;
use utils::ToTypedValue;


impl FenixProfile for Store {
    fn initialize(&mut self) -> Result<(), Error> {
        let mut in_progress = self.begin_transaction()?;
        in_progress.ensure_vocabulary(&Definition {
            name: kw!(:fenix/profile),
            version: 1,
            pre: Definition::no_op,
            post: Definition::no_op,
            attributes: vec![
                // PAGE
                (kw!(:page/url),
                AttributeBuilder::default()
                    .value_type(ValueType::String)
                    .unique(Unique::Value)
                    .index(true)
                    .fulltext(true)
                    .multival(false)
                    .build()),

                // VISIT
                (kw!(:visit/page),
                AttributeBuilder::default()
                    .value_type(ValueType::Ref)
                    .multival(false)
                    .build()),
                (kw!(:visit/when),
                AttributeBuilder::default()
                    .value_type(ValueType::Instant)
                    .multival(false)
                    .build()),

                // URL_META
                (kw!(:url_meta/visit),
                AttributeBuilder::default()
                    .value_type(ValueType::Ref)
                    .multival(false)
                    .build()),
                (kw!(:url_meta/title),
                AttributeBuilder::default()
                    .value_type(ValueType::String)
                    .multival(false)
                    .fulltext(true)
                    .index(true)
                    .build()),
            ],
        })?;
        in_progress.commit()
        .map_err(|e| e.into())
        .and(Ok(()))
    }

    fn record_visit(&mut self, url: String, when: Timespec) -> Result<i64, Error> {
        // Takes a RESERVED lock on the underlying database.
        // We don't want other writers here to affect results of the 'page'
        // lookup, but other readers are fine.
        let mut transaction = self.begin_transaction()?;

        // Look up 'page' by 'url', and insert it if necessary.
        let query = r#"[:find ?eid :where [?eid :page/url ?url]]"#;
        // 'url' will be moved later on into an Arc...
        let args = QueryInputs::with_value_sequence(vec![(var!(?url), url.clone().into())]);
        let rows = transaction.q_once(query, args).into_rel_result()?;
        
        let page_e: TypedValue;

        if rows.is_empty() {
            // 'ok_or_else' for lazy evaluation of err_msg calls.
            let page_url_a = transaction
                .get_entid(&kw!(:page/url))
                .ok_or_else(|| err_msg("expected :page/url"))?;

            let temp_page_e_name = "page";
            let mut page_builder = TermBuilder::new();
            let page_tempid = page_builder.named_tempid(temp_page_e_name.into()).clone();
            page_builder.add(page_tempid, page_url_a, TypedValue::String(Arc::new(url)))?;
            page_e = TypedValue::Ref(*
                transaction.transact_builder(page_builder)?
                .tempids.get(temp_page_e_name)
                .ok_or_else(|| err_msg("expected 'page' in tempids"))?
            );
        } else if rows.row_count() > 1 {
            bail!(err_msg("got more than single 'page' for 'url'"));
        } else {
            // TODO this is just terrible...
            // all I'm trying to do is get a single value out of what's expected to be a single query result.
            let res: Vec<_> = rows.into_iter().map(|r| r[0].clone().val()).collect();
            page_e = res.get(0)
                .ok_or_else(|| err_msg("expected page_e result"))?.clone()
                .ok_or_else(|| err_msg("expected page_e result"))?;
        }
       
        // Finally, insert the visit.
        let mut visit_builder = TermBuilder::new();
        
        let visit_page_a = transaction
            .get_entid(&kw!(:visit/page))
            .ok_or_else(|| err_msg("expected :visit/page"))?;
        let visit_when_a = transaction
            .get_entid(&kw!(:visit/when))
            .ok_or_else(|| err_msg("expected :visit/when"))?;

        let temp_visit_e_name = "visit";
        let temp_visit_e = visit_builder.named_tempid(temp_visit_e_name.into());
        visit_builder.add(temp_visit_e.clone(), visit_page_a, page_e)?;
        visit_builder.add(temp_visit_e.clone(), visit_when_a, when.to_typed_value())?;

        let tempids = transaction.transact_builder(visit_builder)?.tempids;
        let visit_e = tempids
            .get(temp_visit_e_name)
            .ok_or_else(|| err_msg("expected 'visit' in tempids"))?;

        transaction.commit()?;

        Ok(*visit_e)
    }
}

pub trait FenixProfile {
    fn initialize(&mut self) -> Result<(), Error>;
    // TODO use Url type?
    fn record_visit(&mut self, url: String, when: Timespec) -> Result<i64, Error>;
}
 
// TODO
// TLDR; Should be fine to return entid of a new visit.
// What to return? We need to maintain a visit reference in the consuming code, so that observations
// about the visit-related activity may be performed.
// Is that entid? What if we renumber during a sync though? Visits are unlikely to be renumbered.
// What about a visit guid (which we don't have)? Likely more stable than entid, but may still change over time (via merges).
// However, we're talking about a short timeframe of a browsing session for most cases, in which case
// it's unlikely that entid will change. However, that case must be handled when visit observations are tackled.
// When user comes back to a session after a long break (next day, session restore), a new visit will be created.
#[no_mangle]
pub unsafe extern "C" fn fenix_profile_record_visit(manager: *mut Store, url: *const c_char, when: *const time_t) -> ExternResult {
    let manager = &mut*manager;
    let url = c_char_to_string(url);
    let when = Timespec::new(when as i64, 0);

    match manager.record_visit(url.to_string(), when) {
        Ok(visit_e) => ExternResult {
            err: std::ptr::null(),
            ok: string_to_c_char(visit_e.to_string()),
        },
        Err(e) => ExternResult {
            err: string_to_c_char(e.to_string()),
            ok: std::ptr::null(),
        }
    }

    // TODO why does code below require type annotations..?
    // manager
    //     .record_visit(url.to_string(), when)
    //     .map_err(|e| e.into())
    //     .into()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use mentat::QueryBuilder;

    fn get_initialized_store() -> Store {
        let mut db = Store::open("test.db").expect("in-memory store");
        db.initialize().expect("ran initialization");
        db
    }

    // TODO This looks terrible :(
    fn get_all_page_urls(db: &mut Store) -> Vec<TypedValue> {
        QueryBuilder::new(db, r#"[:find ?url :where [?e :page/url ?url]]"#)
            .execute_rel().expect("execute")
            .into_iter()
            .map(|row| row.get(0).expect("").clone().val().expect("typedvalue")).collect()
    }

    #[test]
    fn test_initialization() {
        get_initialized_store();
    }

    #[test]
    fn test_record_visit() {
        let mut db = get_initialized_store();

        assert_eq!(0, get_all_page_urls(&mut db).len());

        db.record_visit("https://www.mozilla.org".to_string(), Timespec::new(1, 0)).expect("recorded visit");

        // TODO this is also terrible...
        let urls = get_all_page_urls(&mut db);
        assert_eq!(urls, vec![TypedValue::String(Arc::new("https://www.mozilla.org".to_string()))]);

        db.record_visit("https://www.mozilla.org".to_string(), Timespec::new(2, 0)).expect("recorded visit");
        assert_eq!(1, get_all_page_urls(&mut db).len());

        db.record_visit("https://www.example.org".to_string(), Timespec::new(3, 0)).expect("recorded visit");
        let urls = get_all_page_urls(&mut db);
        assert_eq!(urls, vec![
            TypedValue::String(Arc::new("https://www.mozilla.org".to_string())),
            TypedValue::String(Arc::new("https://www.example.org".to_string())),
        ]);
    }
}