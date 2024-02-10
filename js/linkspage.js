let sb_default_row = 3;
let sb_row = sb_default_row;
let sb_card_min_width = 250;

let sb_content_w = document.getElementById('content').clientWidth - 25;
while (sb_content_w / sb_row - 10 < sb_card_min_width && sb_row > 1) {
    sb_row -= 1;
}
let sb_card_width = sb_content_w / sb_row - 10;
document.getElementById('link-sb1').style.width = sb_card_width;
if (document.getElementById('link-sb1').style.height < sb_card_width) {
    document.getElementById('link-sb1').style.height = sb_card_width;
}
document.getElementById('link-sb2').style.width = sb_card_width;
if (document.getElementById('link-sb2').style.height < sb_card_width) {
    document.getElementById('link-sb2').style.height = sb_card_width;
}
document.getElementById('link-sb3').style.width = sb_card_width;
if (document.getElementById('link-sb3').style.height < sb_card_width) {
    document.getElementById('link-sb3').style.height = sb_card_width;
}
document.getElementById('link-sb4').style.width = sb_card_width;
if (document.getElementById('link-sb4').style.height < sb_card_width) {
    document.getElementById('link-sb4').style.height = sb_card_width;
}
document.getElementById('link-sb5').style.width = sb_card_width;
if (document.getElementById('link-sb5').style.height < sb_card_width) {
    document.getElementById('link-sb5').style.height = sb_card_width;
}
document.getElementById('link-sb6').style.width = sb_card_width;
if (document.getElementById('link-sb6').style.height < sb_card_width) {
    document.getElementById('link-sb6').style.height = sb_card_width;
}
document.getElementById('link-sb7').style.width = sb_card_width;
if (document.getElementById('link-sb7').style.height < sb_card_width) {
    document.getElementById('link-sb7').style.height = sb_card_width;
}
document.getElementById('link-sb8').style.width = sb_card_width;
if (document.getElementById('link-sb8').style.height < sb_card_width) {
    document.getElementById('link-sb8').style.height = sb_card_width;
}
document.getElementById('link-sb9').style.width = sb_card_width;
if (document.getElementById('link-sb9').style.height < sb_card_width) {
    document.getElementById('link-sb9').style.height = sb_card_width;
}

document.getElementById('link-sbo1').style.width = sb_card_width;
if (document.getElementById('link-sbo1').style.height < sb_card_width) {
    document.getElementById('link-sbo1').style.height = sb_card_width;
}
document.getElementById('link-sbo2').style.width = sb_card_width;
if (document.getElementById('link-sbo2').style.height < sb_card_width) {
    document.getElementById('link-sbo2').style.height = sb_card_width;
}

document.getElementById('link-sbi1').style.width = sb_card_width;
if (document.getElementById('link-sbi1').style.height < sb_card_width) {
    document.getElementById('link-sbi1').style.height = sb_card_width;
}
document.getElementById('link-sbi2').style.width = sb_card_width;
if (document.getElementById('link-sbi2').style.height < sb_card_width) {
    document.getElementById('link-sbi2').style.height = sb_card_width;
}
document.getElementById('link-sbi3').style.width = sb_card_width;
if (document.getElementById('link-sbi3').style.height < sb_card_width) {
    document.getElementById('link-sbi3').style.height = sb_card_width;
}
    
window.addEventListener('resize', function(){
    let sb_default_row = 3;
    let sb_row = sb_default_row;
    let sb_card_min_width = 250;
    
    let sb_content_w = document.getElementById('content').clientWidth - 25;
    while (sb_content_w / sb_row - 10 < sb_card_min_width && sb_row > 1) {
    sb_row -= 1;
    }
    let sb_card_width = sb_content_w / sb_row - 10;
    document.getElementById('link-sb1').style.width = sb_card_width;
    if (document.getElementById('link-sb1').style.height < sb_card_width) {
        document.getElementById('link-sb1').style.height = sb_card_width;
    }
    document.getElementById('link-sb2').style.width = sb_card_width;
    if (document.getElementById('link-sb2').style.height < sb_card_width) {
        document.getElementById('link-sb2').style.height = sb_card_width;
    }
    document.getElementById('link-sb3').style.width = sb_card_width;
    if (document.getElementById('link-sb3').style.height < sb_card_width) {
        document.getElementById('link-sb3').style.height = sb_card_width;
    }
    document.getElementById('link-sb4').style.width = sb_card_width;
    if (document.getElementById('link-sb4').style.height < sb_card_width) {
        document.getElementById('link-sb4').style.height = sb_card_width;
    }
    document.getElementById('link-sb5').style.width = sb_card_width;
    if (document.getElementById('link-sb5').style.height < sb_card_width) {
        document.getElementById('link-sb5').style.height = sb_card_width;
    }
    document.getElementById('link-sb6').style.width = sb_card_width;
    if (document.getElementById('link-sb6').style.height < sb_card_width) {
        document.getElementById('link-sb6').style.height = sb_card_width;
    }
    document.getElementById('link-sb7').style.width = sb_card_width;
    if (document.getElementById('link-sb7').style.height < sb_card_width) {
        document.getElementById('link-sb7').style.height = sb_card_width;
    }
    document.getElementById('link-sb8').style.width = sb_card_width;
    if (document.getElementById('link-sb8').style.height < sb_card_width) {
        document.getElementById('link-sb8').style.height = sb_card_width;
    }
    document.getElementById('link-sb9').style.width = sb_card_width;
    if (document.getElementById('link-sb9').style.height < sb_card_width) {
        document.getElementById('link-sb9').style.height = sb_card_width;
    }

    document.getElementById('link-sbo1').style.width = sb_card_width;
    if (document.getElementById('link-sbo1').style.height < sb_card_width) {
        document.getElementById('link-sbo1').style.height = sb_card_width;
    }
    document.getElementById('link-sbo2').style.width = sb_card_width;
    if (document.getElementById('link-sbo2').style.height < sb_card_width) {
        document.getElementById('link-sbo2').style.height = sb_card_width;
    }

    document.getElementById('link-sbi1').style.width = sb_card_width;
    if (document.getElementById('link-sbi1').style.height < sb_card_width) {
        document.getElementById('link-sbi1').style.height = sb_card_width;
    }
    document.getElementById('link-sbi2').style.width = sb_card_width;
    if (document.getElementById('link-sbi2').style.height < sb_card_width) {
        document.getElementById('link-sbi2').style.height = sb_card_width;
    }
    document.getElementById('link-sbi3').style.width = sb_card_width;
    if (document.getElementById('link-sbi3').style.height < sb_card_width) {
        document.getElementById('link-sbi3').style.height = sb_card_width;
    }
});