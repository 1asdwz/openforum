package com.openforum.repository;

import com.openforum.entity.UserAuthentication;
import com.openforum.entity.enums.AuthLevel;
import com.openforum.entity.enums.AuthStatus;
import com.openforum.entity.enums.AuthType;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.Optional;

public interface UserAuthenticationRepository extends JpaRepository<UserAuthentication, Long> {

    Optional<UserAuthentication> findByUserIdAndType(Long userId, AuthType type);

    List<UserAuthentication> findByUserIdOrderByCreatedAtDesc(Long userId);

    List<UserAuthentication> findByUserIdAndStatus(Long userId, AuthStatus status);

    Page<UserAuthentication> findByStatus(AuthStatus status, Pageable pageable);

    Page<UserAuthentication> findByStatusAndLevel(AuthStatus status, AuthLevel level, Pageable pageable);
}
