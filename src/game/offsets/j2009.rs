use pe_sigscan::pattern;

use crate::game::{July2009, cinput::Action};

use super::{OffsetsProvider, Signature};

impl OffsetsProvider for July2009 {
    // --- client.dll Signatures ---
    fn cinput_create_move_sig(&self) -> Signature {
        pattern![
            0x83, 0xEC, 0x18, 0x55, 0x56, 0x57, 0x8B, 0xF9, 0x8B, 0x0D, _, _, _, _, 0x8B, 0x01,
            0x8B, 0x90, 0x00, 0x02, 0x00, 0x00, 0xFF, 0xD2,
        ] // 0xFC630
    }

    fn chl_input_process_cmd_sig(&self) -> Signature {
        pattern![
            0x8B, 0x0D, _, _, _, _, 0x8B, 0x01, 0x8B, 0x90, 0x58, 0x01, 0x00, 0x00, 0xFF, 0xD2,
            0x84, 0xC0, 0x74, _, 0x8B, 0x44, 0x24, 0x04, 0x0F, 0x57, 0xC0, 0x33, 0xC9, 0x89, 0x48,
            0x24, 0xF3, 0x0F, 0x11, 0x40, 0x18,
        ] // 0xF9670
    }

    fn cviewrender_render_view_sig(&self) -> Signature {
        pattern![
            0x81, 0xEC, 0xBC, 0x01, 0x00, 0x00, 0x53, 0x55, 0x8B, 0xE9, 0x6A, 0x00, 0x8D, 0x8D,
            0xFC, 0x03, 0x00, 0x00, 0xE8, _, _, _, _, 0x8B, 0x0D, _, _, _, _, 0x8B, 0x01, 0x8B,
            0x90, 0x00, 0x02, 0x00, 0x00, 0xFF, 0xD2,
        ] // 0x181120
    }

    fn get_button_bits_sig(&self) -> Signature {
        pattern![
            0x83, 0xEC, 0x08, 0x55, 0x56, 0x8B, 0xF1, 0x8B, 0x0D, _, _, _, _, 0x8B, 0x01, 0x8B,
            0x90, 0x00, 0x02, 0x00, 0x00, 0x57, 0x89, 0x74, 0x24, 0x0C, 0xFF, 0xD2,
        ] // 0xF9710
    }

    fn action_offset(&self, action: Action) -> (usize, usize) {
        match action {
            Action::Attack => (0x106, 0),
            Action::Duck => (0x150, 0),
            Action::Jump => (0x19A, 0),
            Action::Use => (0x278, 0),
            Action::Attack2 => (0x3FE, 0),
            Action::Slowtime => (0x6CE, 0),
        }
    }

    // --- server.dll Signatures ---
    fn accept_input_sig(&self) -> Signature {
        unimplemented!() // only used in P2008
    }
    fn physics_simulate_sig(&self) -> Signature {
        pattern![
            0x83, 0xEC, 0x28, 0x55, 0x57, 0x6A, 0x04, 0x33, 0xED, 0x55, 0x68, _, _, _, _, 0x55,
            0x8B, 0xF9, 0x8B, 0x0D, _, _, _, _, 0x68, _, _, _, _, 0xFF, 0x15, _, _, _, _,
        ] // 0x1C0920
    }

    // Unique to F-Stop
    fn can_be_captured_sig(&self) -> Signature {
        unimplemented!()
    }
    fn scroll_scale_sig(&self) -> Signature {
        unimplemented!()
    }
    fn select_photo_sig(&self) -> Signature {
        unimplemented!()
    }
    fn next_photo_sig(&self) -> Signature {
        unimplemented!()
    }

    // --- engine.dll Signatures ---
    fn p_entity_sig(&self) -> Signature {
        pattern![
            0x56, 0x8B, 0x74, 0x24, 0x08, 0x85, 0xF6, 0x7C, _, 0x3B, 0x35, _, _, _, _, 0x7C, _,
            0x56, 0x68, _, _, _, _,
        ] // 0x98640
    }

    fn key_event_sig(&self) -> Signature {
        pattern![
            0x8B, 0x44, 0x24, 0x04, 0x83, 0xEC, 0x10, 0x57, 0x50, 0x68, 0xD3, 0x03, 0x00, 0x00,
            0x68, _, _, _, _, 0x8D, 0x4C, 0x24, 0x10, 0xE8, _, _, _, _, 0x8B, 0x7C, 0x24, 0x1C,
            0x8A, 0x04, 0xFD, _, _, _, _,
        ] // 0x62790
    }

    fn host_filter_time_sig(&self) -> Signature {
        pattern![
            0x51, 0xF3, 0x0F, 0x10, 0x05, _, _, _, _, 0xF3, 0x0F, 0x58, 0x44, 0x24, 0x08, 0x8B,
            0x0D, _, _, _, _, 0xF3, 0x0F, 0x11, 0x05, _, _, _, _, 0x8B, 0x01, 0x8B, 0x50, 0x20,
            0x53, 0xB3, 0x01, 0xFF, 0xD2,
        ] // 0x511E0
    }

    fn accumulator_sig(&self) -> Signature {
        pattern![
            0xF3, 0x0F, 0x58, 0x05, _, _, _, _, 0xF3, 0x0F, 0x11, 0x05, _, _, _, _
        ] // 0x53467
    }

    // --- engine.dll Offsets ---
    fn key_bindings_offset(&self) -> usize {
        0x23
    }
    fn key_bindings_base_offset(&self) -> usize {
        4
    }
    fn host_filtered_frametime_offset(&self) -> usize {
        0x57
    }

    // --- CViewSetup Offsets ---
    fn cviewsetup_origin_off(&self) -> usize {
        0x2C
    }
    fn cviewsetup_angles_off(&self) -> usize {
        0x38
    }

    // --- CBasePlayer (server) Offsets ---
    fn player_m_flags_off(&self) -> usize {
        0x11C
    }
    fn player_origin_off(&self) -> usize {
        0x27C
    }
    fn player_m_vec_velocity_off(&self) -> usize {
        0x288
    }
    fn player_view_offset_off(&self) -> usize {
        0x340
    }
    fn player_m_h_active_weapon(&self) -> usize {
        unimplemented!() // not used in July 2009
    }
    fn player_m_h_use_entity_off(&self) -> usize {
        0xB9C
    }
    fn player_surface_friction_off(&self) -> usize {
        0x1000
    }
    fn player_h_portal_env_off(&self) -> usize {
        0x1350
    }
    fn pickup_m_h_object_off(&self) -> usize {
        0x23DC
    }

    // --- Flags ---
    fn fl_on_ground(&self) -> u32 {
        0x1
    }
    fn fl_ducking(&self) -> u32 {
        0x4
    }

    // --- CBaseEntity (server) Offsets ---
    fn entity_origin_off(&self) -> usize {
        Self::player_origin_off(self)
    }
    fn entity_classname_off(&self) -> usize {
        0x60
    }
    fn entity_name_off(&self) -> usize {
        0x120
    }
    fn entity_m_ref_ehandle_off(&self) -> usize {
        0x33C
    }
    // F-Stop camera-related
    fn entity_tracker_off(&self) -> usize {
        unimplemented!()
    }
    fn entity_captured_handle_off(&self) -> usize {
        unimplemented!()
    }
    fn entity_controller_off(&self) -> usize {
        unimplemented!()
    }
    fn entity_step_off(&self) -> usize {
        unimplemented!()
    }
    fn entity_in_placement_off(&self) -> usize {
        unimplemented!()
    }

    // --- Controller vtable Slots ---
    fn controller_min_func_slot(&self) -> usize {
        unimplemented!()
    }
    fn controller_max_func_slot(&self) -> usize {
        unimplemented!()
    }
    fn controller_scale_func_slot(&self) -> usize {
        unimplemented!()
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
        0x4D0
    }

    // --- CInput Offsets ---
    fn cinput_slot(&self) -> usize {
        0
    }
    fn cinput_activate_mouse_slot(&self) -> usize {
        18
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
