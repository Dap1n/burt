use crate::game::cinput::Action;

pub mod j2009;
pub mod p2008;

type Signature = &'static [Option<u8>];

/// The interface defining all memory layouts and signatures required by the plugin.
pub trait OffsetsProvider: Send + Sync {
    // --- client.dll Signatures ---
    fn cinput_create_move_sig(&self) -> Signature;
    fn chl_input_process_cmd_sig(&self) -> Signature;
    fn cviewrender_render_view_sig(&self) -> Signature;
    fn get_button_bits_sig(&self) -> Signature;
    fn action_offset(&self, action: Action) -> (usize, usize);

    // --- server.dll Signatures ---
    fn physics_simulate_sig(&self) -> Signature;
    fn scroll_scale_sig(&self) -> Signature;
    fn select_photo_sig(&self) -> Signature;

    // --- engine.dll Signatures ---
    fn p_entity_sig(&self) -> Signature;
    fn key_event_sig(&self) -> Signature;
    fn host_filter_time_sig(&self) -> Signature;
    fn accumulator_sig(&self) -> Signature;
    fn next_photo_sig(&self) -> Signature;

    // --- engine.dll Offsets ---
    fn key_bindings_offset(&self) -> usize;
    fn key_bindings_base_offset(&self) -> usize;
    fn host_filtered_frametime_offset(&self) -> usize;

    // --- CViewSetup Offsets ---
    fn cviewsetup_origin_off(&self) -> usize;
    fn cviewsetup_angles_off(&self) -> usize;

    // --- CBasePlayer (server) Offsets ---
    fn player_m_flags_off(&self) -> usize;
    fn player_origin_off(&self) -> usize;
    fn player_m_vec_velocity_off(&self) -> usize;
    fn player_view_offset_off(&self) -> usize;
    fn player_m_h_active_weapon(&self) -> usize;
    fn player_m_h_use_entity_off(&self) -> usize;
    fn player_surface_friction_off(&self) -> usize;
    fn player_h_portal_env_off(&self) -> usize;
    fn pickup_m_h_object_off(&self) -> usize;

    // --- Flags ---
    fn fl_on_ground(&self) -> u32;
    fn fl_ducking(&self) -> u32;

    // --- CBaseEntity (server) Offsets ---
    fn entity_origin_off(&self) -> usize;
    fn entity_classname_off(&self) -> usize;
    fn entity_name_off(&self) -> usize;
    fn entity_m_ref_ehandle_off(&self) -> usize;
    fn entity_tracker_off(&self) -> usize;
    fn entity_captured_handle_off(&self) -> usize;
    fn entity_controller_off(&self) -> usize;
    fn entity_step_off(&self) -> usize;
    fn entity_in_placement_off(&self) -> usize;

    // --- Controller vtable Slots ---
    fn controller_min_func_slot(&self) -> usize;
    fn controller_max_func_slot(&self) -> usize;
    fn controller_scale_func_slot(&self) -> usize;

    // --- edict_t Offsets ---
    fn edict_portal_internal_off(&self) -> usize;
    fn edict_base_entity_off(&self) -> usize;

    // --- ConVar Offsets/Slots ---
    fn icvar_find_command_slot(&self) -> usize;
    fn convar_set_string_row(&self) -> usize;
    fn convar_set_float_row(&self) -> usize;
    fn convar_set_int_row(&self) -> usize;
    fn convar_float_value_off(&self) -> usize;
    fn convar_int_value_off(&self) -> usize;

    // --- CUserCmd Offsets ---
    fn cusercmd_sequence_number_off(&self) -> usize;
    fn cusercmd_pitch_off(&self) -> usize;
    fn cusercmd_yaw_off(&self) -> usize;
    fn cusercmd_forward_off(&self) -> usize;
    fn cusercmd_side_off(&self) -> usize;

    // --- CProp_Portal Offsets ---
    fn portal_origin_x(&self) -> usize;
    fn portal_origin_y(&self) -> usize;
    fn portal_origin_z(&self) -> usize;
    fn portal_pitch(&self) -> usize;
    fn portal_yaw(&self) -> usize;
    fn portal_roll(&self) -> usize;
    fn portal_h_linked(&self) -> usize;

    // --- CInput Offsets ---
    fn cinput_slot(&self) -> usize;
    fn cinput_activate_mouse_slot(&self) -> usize;
    fn cinput_mouse_active_off(&self) -> usize;
    fn cinput_mouse_off_in_slot(&self) -> usize;
    fn cinput_slot_stride(&self) -> usize;
    fn button_state_stride(&self) -> usize;

    // --- Save Glitch ---
    fn renderview_drawviewmodel(&self) -> i32;
}
