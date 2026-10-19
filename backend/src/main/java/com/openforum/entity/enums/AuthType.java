package com.openforum.entity.enums;

public enum AuthType {
    REAL_NAME,
    ID_CARD,
    GENDER,
    AGE,
    NATIONALITY,
    DRIVER_LICENSE,
    MARRIAGE_CERT,
    PROPERTY_CERT,
    ASSETS,
    /** 学历认证 */
    EDUCATION,
    /** 职业 / 公司 / 行业认证 */
    OCCUPATION,
    /** 所在城市认证 */
    CITY,
    /** 兴趣爱好（无需认证，仅自填）*/
    INTERESTS,
    CUSTOM
}
