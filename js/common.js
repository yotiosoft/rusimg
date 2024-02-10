// gtag
window.dataLayer = window.dataLayer || [];
function gtag(){dataLayer.push(arguments);}
gtag('js', new Date());
gtag('config', 'UA-174934210-1');

// for mobile devices
if ((navigator.userAgent.indexOf('iPhone') > 0) || navigator.userAgent.indexOf('iPod') > 0 || navigator.userAgent.indexOf('Android') > 0) {
    document.write('<meta name="viewport" content="width=device-width, initial-scale=1.0">');
}

//共通パーツ読み込み
$(function() {
    if (window.innerWidth <= 767 || navigator.userAgent.match(/iPhone|Android.+Mobile/)) {
        $("#header").load("/header_m.html");
        $("#footer").load("/footer_m.html");
    }
    else {
        $("#header").load("/header.html");
        $("#footer").load("/footer.html");
    }
    
    window.addEventListener('resize', function(){
        if (window.innerWidth <= 767 || navigator.userAgent.match(/iPhone|Android.+Mobile/)) {
            $("#header").load("/header_m.html");
            $("#footer").load("/footer_m.html");
        }
        else {
            $("#header").load("/header.html");
            $("#footer").load("/footer.html");
        }
    });
});
