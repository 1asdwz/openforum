package com.openforum.service;

import com.openforum.common.JsonSupport;
import com.openforum.dto.user.UpdateUserProfileRequest;
import com.openforum.dto.user.UserProfileDTO;
import com.openforum.entity.UserProfile;
import com.openforum.repository.UserProfileRepository;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.time.LocalDate;
import java.util.List;

@Service
public class UserProfileService {

    private final UserProfileRepository profileRepository;
    private final UserService userService;
    private final JsonSupport jsonSupport;

    public UserProfileService(UserProfileRepository profileRepository,
                              UserService userService,
                              JsonSupport jsonSupport) {
        this.profileRepository = profileRepository;
        this.userService = userService;
        this.jsonSupport = jsonSupport;
    }

    public UserProfileDTO getProfile(Long userId) {
        userService.require(userId);
        UserProfile profile = profileRepository.findByUserId(userId)
                .orElseGet(() -> emptyProfile(userId));
        return toDto(profile);
    }

    @Transactional
    public UserProfileDTO updateProfile(Long userId, UpdateUserProfileRequest req) {
        userService.require(userId);
        UserProfile profile = profileRepository.findByUserId(userId)
                .orElseGet(() -> {
                    UserProfile p = new UserProfile();
                    p.setUserId(userId);
                    return p;
                });

        if (req.gender() != null)        profile.setGender(req.gender().toUpperCase());
        if (req.birthYear() != null)      profile.setBirthYear(req.birthYear());
        if (req.nationality() != null)    profile.setNationality(req.nationality());
        if (req.city() != null)           profile.setCity(req.city());
        if (req.education() != null)      profile.setEducation(req.education().toUpperCase());
        if (req.school() != null)         profile.setSchool(req.school());
        if (req.major() != null)          profile.setMajor(req.major());
        if (req.occupation() != null)     profile.setOccupation(req.occupation());
        if (req.industry() != null)       profile.setIndustry(req.industry());
        if (req.company() != null)        profile.setCompany(req.company());
        if (req.jobLevel() != null)       profile.setJobLevel(req.jobLevel());
        if (req.propertyCount() != null)  profile.setPropertyCount(req.propertyCount());
        if (req.propertyCities() != null) profile.setPropertyCities(jsonSupport.toJson(req.propertyCities()));
        if (req.hasVehicle() != null)     profile.setHasVehicle(req.hasVehicle());
        if (req.incomeLevel() != null)    profile.setIncomeLevel(req.incomeLevel().toUpperCase());
        if (req.maritalStatus() != null)  profile.setMaritalStatus(req.maritalStatus().toUpperCase());
        if (req.interests() != null)      profile.setInterests(jsonSupport.toJson(req.interests()));

        profileRepository.save(profile);
        return toDto(profile);
    }

    public UserProfileDTO toDto(UserProfile profile) {
        int currentYear = LocalDate.now().getYear();
        Integer age = profile.getBirthYear() != null ? currentYear - profile.getBirthYear() : null;
        List<String> propertyCities = jsonSupport.toStringList(profile.getPropertyCities());
        List<String> interests = jsonSupport.toStringList(profile.getInterests());
        return new UserProfileDTO(
                profile.getUserId(),
                profile.getGender(),
                profile.getBirthYear(),
                age,
                profile.getNationality(),
                profile.getCity(),
                profile.getEducation(),
                profile.getSchool(),
                profile.getMajor(),
                profile.getOccupation(),
                profile.getIndustry(),
                profile.getCompany(),
                profile.getJobLevel(),
                profile.getPropertyCount(),
                propertyCities,
                profile.getHasVehicle(),
                profile.getIncomeLevel(),
                profile.getMaritalStatus(),
                interests,
                profile.getUpdatedAt());
    }

    private UserProfile emptyProfile(Long userId) {
        UserProfile p = new UserProfile();
        p.setUserId(userId);
        return p;
    }
}
