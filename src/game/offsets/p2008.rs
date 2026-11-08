use pe_sigscan::pattern;

use super::OffsetsProvider;

/// Zero-sized type representing the F-Stop recreation in Project 2008.
pub struct Project2008;

impl OffsetsProvider for Project2008 {
    // --- client.dll RVAs ---
    fn cinput_create_move_sig(&self) -> &'static [Option<u8>] {
        pattern![
            0x55, 0x8B, 0xEC, 0x83, 0xEC, 0x0C, 0x53, 0x56, 0x57, 0x8B, 0xF9, 0x8B, 0x0D, _, _, _,
            _, 0x8B, 0x01, 0xFF, 0x90, 0xFC, 0x01, 0x00, 0x00, 0x8B, 0x0D, _, _, _, _, 0x8B, 0xD8,
            0xF6, 0xC1, 0x01, 0x0F, 0x85, _, _, _, _,
        ] // 0x2BB220
    }

    fn chl_input_process_cmd_sig(&self) -> &'static [Option<u8>] {
        pattern![
            0x55, 0x8B, 0xEC, 0x8B, 0x0D, _, _, _, _, 0x8B, 0x01, 0x8B, 0x80, 0x54, 0x01, 0x00,
            0x00, 0xFF, 0xD0, 0x84, 0xC0, 0x74, _, 0x8B, 0x45, 0x08, 0xC7, 0x40, 0x24, 0x00, 0x00,
            0x00, 0x00, 0xC7, 0x40, 0x18, 0x00, 0x00, 0x00, 0x00, 0xC7, 0x40, 0x1C, 0x00, 0x00,
            0x00, 0x00,
        ] // 0x2BA410
    }

    fn cviewrender_render_view_sig(&self) -> &'static [Option<u8>] {
        pattern![
            0x55, 0x8B, 0xEC, 0x81, 0xEC, 0x30, 0x02, 0x00, 0x00, 0xA1, _, _, _, _, 0x33, 0xC5,
            0x89, 0x45, 0xFC, 0x8B, 0x45, 0x0C, 0x53, 0x8B, 0x5D, 0x08, 0x57, 0x8B, 0xF9, 0x89,
            0x9D, 0xF4, 0xFE, 0xFF, 0xFF, 0x6A, 0x00, 0x89, 0xBD, 0xEC, 0xFE, 0xFF, 0xFF, 0x89,
            0x85, 0xF0, 0xFE, 0xFF, 0xFF, 0x8D, 0x8F, 0x28, 0x02, 0x00, 0x00, 0xE8, _, _, _, _,
            0x8B, 0x0D, _, _, _, _, 0x8B, 0x01, 0xFF, 0x90, 0xFC, 0x01, 0x00, 0x00,
        ] // 0x351b30
    }

    fn get_button_bits_sig(&self) -> &'static [Option<u8>] {
        pattern![
            0x55, 0x8B, 0xEC, 0x83, 0xEC, 0x0C, 0x53, 0x8B, 0xD9, 0x8B, 0x0D, _, _, _, _, 0x56,
            0x57, 0x89, 0x5D, 0xF8, 0x8B, 0x01, 0xFF, 0x90, 0xFC, 0x01, 0x00, 0x00,
        ] // 0x2BB4F0
    }

    // --- server.dll RVAs ---
    fn physics_simulate_sig(&self) -> &'static [Option<u8>] {
        pattern![
            0x55, 0x8B, 0xEC, 0x83, 0xEC, 0x34, 0x53, 0x6A, 0x04, 0x6A, 0x00, 0x68, _, _, _, _,
            0x8B, 0xD9, 0x8B, 0x0D, _, _, _, _, 0x6A, 0x00, 0x68, _, _, _, _, 0x89, 0x5D, 0xF8,
            0xFF, 0x15, _, _, _, _, 0x8B, 0x83, 0x58, 0x01, 0x00, 0x00, 0x83, 0xF8, 0xFF, 0x74, _,
        ] // 0x2D4C10
    }

    // --- engine.dll RVAs ---
    fn p_entity_sig(&self) -> &'static [Option<u8>] {
        pattern![
            0x56, 0x8B, 0x74, 0x24, 0x08, 0x85, 0xF6, 0x7C, _, 0x3B, 0x35, _, _, _, _, 0x7C, _,
            0x56, 0x68, _, _, _, _,
        ] // 0x9AAC0
    }

    fn key_event_sig(&self) -> &'static [Option<u8>] {
        pattern![
            0x83, 0xEC, 0x10, 0x55, 0x8B, 0x6C, 0x24, 0x18, 0x56, 0x8B, 0x75, 0x08, 0x57, 0x8D,
            0x46, 0x8E, 0x33, 0xFF, 0x3D, 0xBF, 0x00, 0x00, 0x00, 0x76, _,
        ] // 0x63670
    }

    fn host_filter_time_sig(&self) -> &'static [Option<u8>] {
        pattern![
            0x51, 0xF3, 0x0F, 0x10, 0x05, _, _, _, _, 0xF3, 0x0F, 0x58, 0x44, 0x24, 0x08, 0x8B,
            0x0D, _, _, _, _, 0xF3, 0x0F, 0x11, 0x05, _, _, _, _, 0x8B, 0x01, 0x8B, 0x50, 0x20,
            0x53, 0xB3, 0x01, 0xFF, 0xD2,
        ] // 0x52060
    }

    fn accumulator_sig(&self) -> &'static [Option<u8>] {
        pattern![
            0xF3, 0x0F, 0x58, 0x05, _, _, _, _, 0xF3, 0x0F, 0x11, 0x05, _, _, _, _,
        ] // 0x541f0
    }

    // --- engine.dll Offsets ---
    fn key_bindings_offset(&self) -> usize {
        0x81
    }
    fn key_bindings_base_offset(&self) -> usize {
        4
    }
    fn host_filtered_frametime_offset(&self) -> usize {
        0x4A
    }

    // --- CViewSetup Offsets ---
    fn cviewsetup_origin_off(&self) -> usize {
        0x60
    }
    fn cviewsetup_angles_off(&self) -> usize {
        0x6C
    }

    // --- CBasePlayer (server) Offsets ---
    fn player_m_flags_off(&self) -> usize {
        0x120
    }
    fn player_origin_off(&self) -> usize {
        0x280
    }
    fn player_m_vec_velocity_off(&self) -> usize {
        0x28C
    }
    fn player_view_offset_off(&self) -> usize {
        0x358
    }
    fn player_m_h_use_entity_off(&self) -> usize {
        0xC34
    }
    fn player_surface_friction_off(&self) -> usize {
        0x10A8
    }
    fn player_h_portal_env_off(&self) -> usize {
        0x15E8
    }
    fn pickup_m_h_object_off(&self) -> usize {
        unimplemented!()
    }

    // --- Flags ---
    fn fl_on_ground(&self) -> u32 {
        0x1
    }
    fn fl_ducking(&self) -> u32 {
        0x2
    }

    // --- CBaseEntity (server) Offsets ---
    fn entity_origin_off(&self) -> usize {
        Self::player_origin_off(self)
    }
    fn entity_classname_off(&self) -> usize {
        0x60
    }
    fn entity_name_off(&self) -> usize {
        0x124
    }
    fn entity_m_ref_ehandle_off(&self) -> usize {
        0x340
    }

    // --- edict_t Offsets ---
    fn edict_portal_internal_off(&self) -> usize {
        0x08
    }
    fn edict_base_entity_off(&self) -> usize {
        0x0C
    }

    // --- ConVar Offsets/Slots ---
    fn icvar_find_command_slot(&self) -> usize {
        14
    }
    fn convar_set_string_row(&self) -> usize {
        12
    }
    fn convar_set_float_row(&self) -> usize {
        13
    }
    fn convar_set_int_row(&self) -> usize {
        14
    }
    fn convar_float_value_off(&self) -> usize {
        0x2C
    }
    fn convar_int_value_off(&self) -> usize {
        0x30
    }

    // --- CUserCmd Offsets ---
    fn cusercmd_sequence_number_off(&self) -> usize {
        0x04
    }
    fn cusercmd_pitch_off(&self) -> usize {
        0x0C
    }
    fn cusercmd_yaw_off(&self) -> usize {
        0x10
    }
    fn cusercmd_forward_off(&self) -> usize {
        0x18
    }
    fn cusercmd_side_off(&self) -> usize {
        0x1C
    }

    // --- CProp_Portal Offsets ---
    fn portal_origin_x(&self) -> usize {
        0x1C
    }
    fn portal_origin_y(&self) -> usize {
        0x20
    }
    fn portal_origin_z(&self) -> usize {
        0x24
    }
    fn portal_pitch(&self) -> usize {
        0x308
    }
    fn portal_yaw(&self) -> usize {
        0x30C
    }
    fn portal_roll(&self) -> usize {
        0x310
    }
    fn portal_h_linked(&self) -> usize {
        0x51C
    }

    // --- CInput Offsets ---
    fn cinput_slot(&self) -> usize {
        0
    }
    fn cinput_mouse_active_off(&self) -> usize {
        0x6
    }
    fn cinput_mouse_off_in_slot(&self) -> usize {
        0x2C
    }
    fn cinput_slot_stride(&self) -> usize {
        0xC8
    }
    fn button_state_stride(&self) -> usize {
        0xC
    }

    // --- Save Glitch ---
    fn renderview_drawviewmodel(&self) -> i32 {
        1
    }
}
