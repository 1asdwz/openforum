package com.openforum.entity;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.PrePersist;
import jakarta.persistence.PreUpdate;
import jakarta.persistence.Table;

import java.time.LocalDateTime;

/**
 * 用户扩展档案（与 users 表 1:1，可选填写）。
 * 字段值与认证等级分开存储：字段值在此表，认证等级在 user_authentications 表。
 */
@Entity
@Table(name = "user_profiles")
public class UserProfile {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(name = "user_id", nullable = false, unique = true)
    private Long userId;

    // ── 基本信息 ──────────────────────────────────────────
    /** MALE / FEMALE / OTHER */
    @Column(name = "gender", length = 20)
    private String gender;

    /** 出生年份，运行时按当年计算年龄 */
    @Column(name = "birth_year")
    private Integer birthYear;

    @Column(name = "nationality", length = 80)
    private String nationality;

    @Column(name = "city", length = 80)
    private String city;

    // ── 学历 ─────────────────────────────────────────────
    /** HIGHSCHOOL / ASSOCIATE / BACHELOR / MASTER / PHD */
    @Column(name = "education", length = 20)
    private String education;

    @Column(name = "school", length = 120)
    private String school;

    @Column(name = "major", length = 120)
    private String major;

    // ── 职业 ─────────────────────────────────────────────
    @Column(name = "occupation", length = 60)
    private String occupation;

    @Column(name = "industry", length = 60)
    private String industry;

    @Column(name = "company", length = 120)
    private String company;

    @Column(name = "job_level", length = 60)
    private String jobLevel;

    // ── 资产 ─────────────────────────────────────────────
    /** 0 = 无 / 1 = 1套 / 2 = 2套及以上 */
    @Column(name = "property_count")
    private Integer propertyCount;

    /** JSON 数组，如 ["上海","北京"] */
    @Column(name = "property_cities", columnDefinition = "TEXT")
    private String propertyCities;

    @Column(name = "has_vehicle")
    private Boolean hasVehicle;

    /** LOW / MEDIUM / HIGH */
    @Column(name = "income_level", length = 20)
    private String incomeLevel;

    // ── 个人状况 ──────────────────────────────────────────
    /** SINGLE / MARRIED / DIVORCED */
    @Column(name = "marital_status", length = 20)
    private String maritalStatus;

    /** JSON 标签数组，如 ["股票投资","房产"] */
    @Column(name = "interests", columnDefinition = "TEXT")
    private String interests;

    @Column(name = "updated_at", nullable = false)
    private LocalDateTime updatedAt;

    @PrePersist
    @PreUpdate
    void touch() {
        updatedAt = LocalDateTime.now();
    }

    // ── Getters / Setters ─────────────────────────────────

    public Long getId() { return id; }
    public void setId(Long id) { this.id = id; }

    public Long getUserId() { return userId; }
    public void setUserId(Long userId) { this.userId = userId; }

    public String getGender() { return gender; }
    public void setGender(String gender) { this.gender = gender; }

    public Integer getBirthYear() { return birthYear; }
    public void setBirthYear(Integer birthYear) { this.birthYear = birthYear; }

    public String getNationality() { return nationality; }
    public void setNationality(String nationality) { this.nationality = nationality; }

    public String getCity() { return city; }
    public void setCity(String city) { this.city = city; }

    public String getEducation() { return education; }
    public void setEducation(String education) { this.education = education; }

    public String getSchool() { return school; }
    public void setSchool(String school) { this.school = school; }

    public String getMajor() { return major; }
    public void setMajor(String major) { this.major = major; }

    public String getOccupation() { return occupation; }
    public void setOccupation(String occupation) { this.occupation = occupation; }

    public String getIndustry() { return industry; }
    public void setIndustry(String industry) { this.industry = industry; }

    public String getCompany() { return company; }
    public void setCompany(String company) { this.company = company; }

    public String getJobLevel() { return jobLevel; }
    public void setJobLevel(String jobLevel) { this.jobLevel = jobLevel; }

    public Integer getPropertyCount() { return propertyCount; }
    public void setPropertyCount(Integer propertyCount) { this.propertyCount = propertyCount; }

    public String getPropertyCities() { return propertyCities; }
    public void setPropertyCities(String propertyCities) { this.propertyCities = propertyCities; }

    public Boolean getHasVehicle() { return hasVehicle; }
    public void setHasVehicle(Boolean hasVehicle) { this.hasVehicle = hasVehicle; }

    public String getIncomeLevel() { return incomeLevel; }
    public void setIncomeLevel(String incomeLevel) { this.incomeLevel = incomeLevel; }

    public String getMaritalStatus() { return maritalStatus; }
    public void setMaritalStatus(String maritalStatus) { this.maritalStatus = maritalStatus; }

    public String getInterests() { return interests; }
    public void setInterests(String interests) { this.interests = interests; }

    public LocalDateTime getUpdatedAt() { return updatedAt; }
    public void setUpdatedAt(LocalDateTime updatedAt) { this.updatedAt = updatedAt; }
}
