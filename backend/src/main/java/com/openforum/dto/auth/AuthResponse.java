package com.openforum.dto.auth;

import com.openforum.dto.user.UserDTO;

public record AuthResponse(String token, UserDTO user) {
}
