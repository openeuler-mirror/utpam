/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

use crate::common::*;
use crate::utpam::*;
use crate::utpam_item::{utpam_get_item, utpam_set_item};
use crate::utpam_vprompt::pam_vprompt;
use crate::{pam_error, pam_prompt};
use std::any::Any;
const PROMPT: &str = "Password: ";
const PROMPT_CURRENT_NOARG: &str = "Current password: ";
const PROMPT_NEW_NOARG: &str = "New password: ";
const PROMPT_RETYPE_NOARG: &str = "Retype new password: ";

const PAM_GETAUTHTOK_NOVERIFY: u32 = 1;

pub fn get_option(utpamh: &mut Box<UtpamHandle>, option: &str) -> Option<String> {
    if option.is_empty() || utpamh.mod_argc == 0 || utpamh.mod_argv.is_empty() {
        return None;
    }
    // 遍历参数列表
    for arg in &utpamh.mod_argv {
        if let Some((key, value)) = arg.split_once('=') {
            if key == option {
                return Some(value.to_string());
            }
        } else if arg == option {
            return Some(String::new());
        }
    }

    None
}

fn utpam_get_authtok_internal(
    utpamh: &mut Box<UtpamHandle>,
    item: i8,
    authtok: &mut Option<String>,
    prompt: Option<String>,
    flags: u32,
) -> u8 {
    let mut chpass: i32 = 0;
    let mut authtok_type: Option<String> = None;
    let mut retval: u8;
    let mut prevauthtok: Box<dyn Any> = Box::new(());
    let resp: [String; 2] = [String::new(), String::new()];

    if authtok.is_none() {
        return PAM_SYSTEM_ERR;
    };

    if utpamh.choice == PAM_CHAUTHTOK {
        if item == PAM_AUTHTOK {
            chpass = 1;
            if flags & PAM_GETAUTHTOK_NOVERIFY == 0 {
                chpass += 1;
            }
        }

        authtok_type = get_option(utpamh, "authtok_type");

        match authtok_type {
            Some(ref tok) => {
                utpam_set_item(utpamh, PAM_AUTHTOK_TYPE, Some(Box::new(tok.clone())));
            }
            None => {
                let mut tok_type: Box<dyn Any> = Box::new(());
                retval = utpam_get_item(utpamh, PAM_AUTHTOK_TYPE, &mut tok_type);

                //utpam_get_item是克隆utpamh.authtok_type，其类型是String，所以解引用时要使用String
                match tok_type.downcast_ref::<String>() {
                    Some(s) => authtok_type = Some(s.to_string()),
                    None => authtok_type = None,
                }

                if retval != PAM_SUCCESS {
                    authtok_type = None;
                }
            }
        }
    }

    retval = utpam_get_item(utpamh, item, &mut prevauthtok);
    //prevauthtok为旧密码，通常是PAM_AUTHTOK 或 PAM_OLDAUTHTOK。对应类型是Option<String>
    if retval == PAM_SUCCESS {
        if let Some(pre) = prevauthtok.downcast_ref::<Option<String>>() {
            authtok.clone_from(pre);
            return PAM_SUCCESS;
        }
    } else if get_option(utpamh, "use_first_pass").is_some()
        || (chpass == 1 && get_option(utpamh, "use_authtok").is_some())
    {
        match prevauthtok.downcast_ref::<Option<String>>() {
            Some(_) => {
                return retval;
            }
            None => {
                if chpass == 1 {
                    return PAM_AUTHTOK_ERR;
                } else {
                    return PAM_AUTH_ERR;
                }
            }
        }
    }

    if let Some(prompt) = prompt {
        retval = pam_prompt!(utpamh, PAM_PROMPT_ECHO_OFF, resp[0].clone(), "{}", prompt) as u8;
        if retval == PAM_SUCCESS && chpass > 1 && !resp[0].is_empty() {
            retval = pam_prompt!(
                utpamh,
                PAM_PROMPT_ECHO_OFF,
                resp[1].clone(),
                "Retype {}",
                prompt
            ) as u8;
        }
    } else if chpass == 1 {
        utpamh.authtok_verified = 0;
        retval = match authtok_type {
            Some(ref tok) => pam_prompt!(
                utpamh,
                PAM_PROMPT_ECHO_OFF,
                resp[1].clone(),
                "New {} password: ",
                tok
            ) as u8,
            None => pam_prompt!(
                utpamh,
                PAM_PROMPT_ECHO_OFF,
                resp[1].clone(),
                "{}",
                PROMPT_NEW_NOARG
            ) as u8,
        }
    } else if item == PAM_OLDAUTHTOK {
        retval = match authtok_type {
            Some(ref tok) => pam_prompt!(
                utpamh,
                PAM_PROMPT_ECHO_OFF,
                resp[0].clone(),
                "Current {} password: ",
                tok
            ) as u8,
            None => pam_prompt!(
                utpamh,
                PAM_PROMPT_ECHO_OFF,
                resp[0].clone(),
                "{}",
                PROMPT_CURRENT_NOARG
            ) as u8,
        }
    } else {
        retval = pam_prompt!(utpamh, PAM_PROMPT_ECHO_OFF, resp[0].clone(), "{}", PROMPT) as u8;
    }

    if (retval != PAM_SUCCESS || resp[0].is_empty() || (chpass > 1 && resp[1].is_empty()))
        && chpass == 1
    {
        println!("Password change has been aborted.");
        return PAM_AUTHTOK_ERR;
    }

    if chpass > 1 && resp[0] != resp[1] {
        return PAM_TRY_AGAIN;
    }

    retval = utpam_set_item(utpamh, item, Some(Box::new(resp[0].clone())));
    if retval != PAM_SUCCESS {
        return retval;
    }

    if chpass > 1 {
        utpamh.authtok_verified = 1;
    }

    let mut value: Box<dyn Any> = Box::new(());
    retval = utpam_get_item(utpamh, item, &mut value);

    if let Some(aut) = value.downcast_ref::<String>() {
        *authtok = Some(aut.to_string());
    }

    retval
}

pub fn utpam_get_authtok(
    utpamh: &mut Box<UtpamHandle>,
    item: i8,
    authtok: &mut Option<String>,
    prompt: Option<String>,
) -> u8 {
    utpam_get_authtok_internal(utpamh, item, authtok, prompt, 0)
}

pub fn utpam_get_authtok_noverify(
    utpamh: &mut Box<UtpamHandle>,
    authtok: &mut Option<String>,
    prompt: Option<String>,
) -> u8 {
    utpam_get_authtok_internal(
        utpamh,
        PAM_AUTHTOK,
        authtok,
        prompt,
        PAM_GETAUTHTOK_NOVERIFY,
    )
}

pub fn utpam_get_authtok_verify(
    utpamh: &mut Box<UtpamHandle>,
    authtok: &mut Option<String>,
    prompt: Option<String>,
) -> u8 {
    let mut retval: u8;
    let resp: String = String::new();
    let mut authtok_type: Option<String>;

    if authtok.is_none() || utpamh.choice != PAM_CHAUTHTOK {
        return PAM_SYSTEM_ERR;
    };

    if utpamh.authtok_verified != 0 {
        let mut value: Box<dyn Any> = Box::new(());
        retval = utpam_get_item(utpamh, PAM_AUTHTOK, &mut value);

        //utpam_get_item是克隆utpamh.authtok_type，其类型是String，所以解引用时要使用String
        match value.downcast_ref::<String>() {
            Some(s) => *authtok = Some(s.to_string()),
            None => return retval,
        }
    }

    if let Some(p) = prompt {
        retval = pam_prompt!(utpamh, PAM_PROMPT_ECHO_OFF, resp.clone(), "Retype {}", p) as u8;
    } else {
        let mut value: Box<dyn Any> = Box::new(());
        retval = utpam_get_item(utpamh, PAM_AUTHTOK_TYPE, &mut value);

        //map方法：如果Option是Some返回s.to_string()，否则返回None
        authtok_type = value.downcast_ref::<String>().map(|s| s.to_string());

        if retval != PAM_SUCCESS {
            authtok_type = None;
        }

        retval = match authtok_type {
            Some(ref tok) => pam_prompt!(
                utpamh,
                PAM_PROMPT_ECHO_OFF,
                resp.clone(),
                "Retype new {} password: ",
                tok
            ) as u8,
            None => pam_prompt!(
                utpamh,
                PAM_PROMPT_ECHO_OFF,
                resp.clone(),
                "{}",
                PROMPT_RETYPE_NOARG
            ) as u8,
        }
    }

    if retval != PAM_SUCCESS || resp.is_empty() {
        utpam_set_item(utpamh, PAM_AUTHTOK, None);
        pam_error!(utpamh, "Password change has been aborted.",);
        return PAM_AUTHTOK_ERR;
    }

    if let Some(tok) = authtok {
        if *tok != resp {
            utpam_set_item(utpamh, PAM_AUTHTOK, None);
            pam_error!(utpamh, "Password change has been aborted.",);
            return PAM_TRY_AGAIN;
        }
    }

    retval = utpam_set_item(utpamh, PAM_AUTHTOK, Some(Box::new(resp)));

    if retval != PAM_SUCCESS {
        return retval;
    }
    utpamh.authtok_verified = 1;

    let mut value: Box<dyn Any> = Box::new(());
    retval = utpam_get_item(utpamh, PAM_AUTHTOK, &mut value);

    if let Some(aut) = value.downcast_ref::<String>() {
        *authtok = Some(aut.to_string());
    }

    retval
}
