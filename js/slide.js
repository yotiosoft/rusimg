$(function() {
    $('.slide').each(function() {
      // スライド（画像）の数を取得
      var $slides = $(this).find('p'),
          slideNum = $slides.length,
          currentIdx = 0; // 何番目か
    
      // 最初の画像をフェードイン
      $(".slide p").eq(currentIdx).fadeIn();
    
      // 8秒後に次のスライドを表示
      setTimeout(dispNextSlide, 8000);
    
      // 次のスライドを表示するメソッド
      function dispNextSlide() {
        var nextIdx = currentIdx + 1;
    
        // 最後のスライドの場合ははじめに戻る
        if (nextIdx > (slideNum - 1)) {
          nextIdx = 0
        }
    
        // 現在のスライドをフェードアウト
        $(".slide p").eq(currentIdx).fadeOut();
    
        // 次のスライドをフェードイン
        $(".slide p").eq(nextIdx).fadeIn();
    
        // インデックスを更新
        currentIdx = nextIdx;

        setTimeout(dispNextSlide, 8000);
      }
    });
});