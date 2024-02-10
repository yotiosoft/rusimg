$(function($){
    var content = $("#content");
    var sidebar = $("#sidebar");
    var sticked = $("#sticked");

    // サイドバーの位置
    var sidebar_top = sidebar.offset().top;
    // 固定するコンテンツの元々の位置
    var sticked_original_top = sticked.offset().top;
    // 固定するコンテンツの高さ
    var sticked_height = sticked.height();

    var mode = window.innerWidth > 767 ? "pc" : "mobile";

    $(window).on('resize', function(){          // リサイズ時
        if (window.innerWidth <= 767 || navigator.userAgent.match(/iPhone|Android.+Mobile/)) {
            mode = "mobile";
        }
        else {
            mode = "pc";
        }

        sidebar_top = sidebar.offset().top;
        sticked_original_top = sticked.offset().top;
        sticked_height = sticked.height();

        put_sticked();
    });

    function put_sticked() {
        if (mode == "mobile") {
            sidebar.css({'max-width': '100%'});
            sticked.css({'position': 'static', 'width': '100%'});

            console.log("change");

            return;
        }

        sidebar.css({'max-width': '15%'});

        // 現在の位置
        var client_hight = 0;
        var scrollTop = $(document).scrollTop() + client_hight;
        // メインコンテンツ最後尾
        var content_bottom = content.offset().top + content.height();

        if ((scrollTop >= sticked_original_top) && (scrollTop < content_bottom - sticked_height)){
            //console.log(`scrollTop: ${scrollTop}, sticked_original_top: ${sticked_original_top}, content_bottom: ${content_bottom}, sticked_height: ${sticked_height}`);
            // 現在位置が、初期位置より下で、メインコンテンツ最後尾より上なら、画面上部にサイドバーを固定
            sticked.css({
                'position': 'fixed',
                'top': client_hight + 30,
                'width': '15%'
            });
        } else if(scrollTop >= content_bottom - sticked_height){
            // 現在位置がメインコンテンツ最後尾より下なら、メインコンテンツ最後尾にサイドバーを位置させる
            sticked.css({
                'position': 'absolute',
                'top': content_bottom - sticked_height - sidebar_top,
                'width': '100%'
            });
        } else {
            // 現在位置が初期位置より上なら、何もしない
            sticked.css({'position': 'static', 'width': '100%'});
        }
    }

    $(window).on('load scroll', put_sticked);
});
