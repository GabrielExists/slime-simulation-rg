// FIXME(eddyb) update/review these lints.
//
// BEGIN - Embark standard lints v0.4
// do not change or add/remove here, but one can add exceptions after this section
// for more info see: <https://github.com/EmbarkStudios/rust-ecosystem/issues/59>
//#![deny(unsafe_code)]  // quite a bit of unsafe with wgpu still, would be nice to remove
#![warn(
clippy::all,
clippy::await_holding_lock,
clippy::char_lit_as_u8,
clippy::checked_conversions,
clippy::dbg_macro,
clippy::debug_assert_with_mut_call,
clippy::doc_markdown,
clippy::empty_enum,
clippy::enum_glob_use,
clippy::exit,
clippy::expl_impl_clone_on_copy,
clippy::explicit_deref_methods,
clippy::explicit_into_iter_loop,
clippy::fallible_impl_from,
clippy::filter_map_next,
clippy::float_cmp_const,
clippy::fn_params_excessive_bools,
clippy::if_let_mutex,
clippy::implicit_clone,
clippy::imprecise_flops,
clippy::inefficient_to_string,
clippy::invalid_upcast_comparisons,
clippy::large_types_passed_by_value,
clippy::let_unit_value,
clippy::linkedlist,
clippy::lossy_float_literal,
clippy::macro_use_imports,
clippy::manual_ok_or,
clippy::map_err_ignore,
clippy::map_flatten,
clippy::map_unwrap_or,
clippy::match_on_vec_items,
clippy::match_same_arms,
clippy::match_wildcard_for_single_variants,
clippy::mem_forget,
clippy::mut_mut,
clippy::mutex_integer,
clippy::needless_borrow,
clippy::needless_continue,
clippy::option_option,
clippy::path_buf_push_overwrite,
clippy::ptr_as_ptr,
clippy::ref_option_ref,
clippy::rest_pat_in_fully_bound_structs,
clippy::same_functions_in_if_condition,
clippy::semicolon_if_nothing_returned,
clippy::string_add_assign,
clippy::string_add,
clippy::string_lit_as_bytes,
clippy::string_to_string,
clippy::todo,
clippy::trait_duplication_in_bounds,
clippy::unimplemented,
clippy::unnested_or_patterns,
clippy::unused_self,
clippy::useless_transmute,
clippy::verbose_file_reads,
clippy::zero_sized_map_values,
future_incompatible,
nonstandard_style,
rust_2018_idioms
)]
#![feature(noop_waker)]


mod window;
mod program;
mod slot_render;
mod slot_agents;
mod slot_diffuse;
mod configuration;
mod slot_egui;
mod configuration_menu;
mod slot_mouse;

fn main() {
    window::run();
}
