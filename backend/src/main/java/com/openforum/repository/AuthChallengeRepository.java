package com.openforum.repository;

import com.openforum.entity.AuthChallenge;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.Optional;

public interface AuthChallengeRepository extends JpaRepository<AuthChallenge, String> {

    Optional<AuthChallenge> findByIdAndUserId(String id, Long userId);
}
