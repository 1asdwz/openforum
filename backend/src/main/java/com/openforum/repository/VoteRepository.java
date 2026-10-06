package com.openforum.repository;

import com.openforum.entity.Vote;
import com.openforum.entity.enums.VoteTargetType;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.Optional;

public interface VoteRepository extends JpaRepository<Vote, Long> {

    Optional<Vote> findByUserIdAndTargetTypeAndTargetId(Long userId, VoteTargetType targetType, Long targetId);

    void deleteByTargetTypeAndTargetId(VoteTargetType targetType, Long targetId);
}
