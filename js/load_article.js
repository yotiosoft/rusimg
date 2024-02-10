function put_cards(max_articles) {
    console.log(max_articles);
    var articles_wrap = document.getElementById("articles_wrap");
    
    $.getJSON("/articles/articles.json", function(data){
        for (let i=0; i<Math.min(data.length, max_articles); i++) {
            /* カードを追加 */
            var e_blockcard = document.createElement('div');
            e_blockcard.className = 'blockcard';
            
            var e_blockcard_link = document.createElement('a');
            e_blockcard_link.id = 'blockcard_link';
    
            var e_blockcard_image_wrap = document.createElement('div');
            e_blockcard_image_wrap.className = 'blockcard_image_wrap';

            var e_blockcard_image = document.createElement('picture');
            var e_blockcard_image_source = document.createElement('source');
            e_blockcard_image_source.className = 'blockcard_image';
            e_blockcard_image.append(e_blockcard_image_source);
            var e_blockcard_image_img = document.createElement('img');
            e_blockcard_image_img.className = 'blockcard_image';
            e_blockcard_image.append(e_blockcard_image_img);
            e_blockcard_image_wrap.appendChild(e_blockcard_image);
            e_blockcard_link.appendChild(e_blockcard_image_wrap);
            e_blockcard.appendChild(e_blockcard_link);
    
            var e_blockcard_absolute = document.createElement('div');
            e_blockcard_absolute.className = 'blockcard_absolute';
    
            var e_blockcard_title = document.createElement('p');
            e_blockcard_title.className = 'blockcard_small-title';
            e_blockcard_title.id = 'blockcard_title_id';
            e_blockcard_absolute.appendChild(e_blockcard_title);
    
            var e_blockcard_tags = document.createElement('p');
            e_blockcard_tags.className = 'blockcard_tags';
            e_blockcard_tags.id = 'blockcard_tags_id';
            e_blockcard_tags.style = 'display: flex; flex-wrap: wrap; align-items: center;'
            e_blockcard_absolute.appendChild(e_blockcard_tags);
    
            var e_blockcard_content = document.createElement('p');
            e_blockcard_content.className = 'blockcard_content';
            e_blockcard_content.id = 'blockcard_content_id';
            e_blockcard_absolute.appendChild(e_blockcard_content);
    
            var e_blockcard_button_wrap = document.createElement('span');
            e_blockcard_button_wrap.className = 'blockcard_button_wrap';
    
            var e_blockcard_button = document.createElement('span');
            e_blockcard_button.className = 'blockcard_button';
    
            var e_readmore_button = document.createElement('a');
            e_readmore_button.className = 'readmore_button';
            e_readmore_button.style = 'color:#fff;" id="blockcard_read_button';
            e_readmore_button.textContent = '詳しく見る';
            e_blockcard_button.appendChild(e_readmore_button);
            e_blockcard_button_wrap.appendChild(e_blockcard_button);
            e_blockcard_absolute.appendChild(e_blockcard_button_wrap);
            e_blockcard.appendChild(e_blockcard_absolute);
            
            articles_wrap.appendChild(e_blockcard);
    
            /* データ代入 */
            e_blockcard_link.href = data[i].link;
            e_readmore_button.href = data[i].link;
            e_blockcard_image_source.srcset = data[i].image_webp;
            e_blockcard_image_img.src = data[i].image;
            e_blockcard_content.textContent = "読み込み中...";
    
            get_data_for_card(data[i], e_blockcard_content, e_blockcard_tags, e_blockcard_title);
        }
    });
}

function get_data_for_card(data, arg_blockcard_content, arg_blockcard_tags, arg_blockcard_title) {
    const formatter = new Intl.NumberFormat('ja', {
        minimumIntegerDigits: 2, 
        useGrouping: false
    });

    const doc = fetch(data.link)
    .then((res) => res.text())
    .then((text) => {
        return new DOMParser().parseFromString(text, "text/html");
        })
    .then((doc) => {
        arg_blockcard_content.innerHTML = doc.getElementById("article_summary").innerText;
        arg_blockcard_tags.innerHTML = doc.getElementById("article_tags").innerHTML +
            "<p style=\"width: fit-content; margin-left: 5px; font-size: 14px;\"><b>" + data.year+"."+formatter.format(data.month)+"."+formatter.format(data.day) + "</b></p>";
        arg_blockcard_title.innerHTML = doc.getElementById("article_title").innerText;
    }
    );
}

function make_articles_table(start, total) {
    const formatter = new Intl.NumberFormat('ja', {
        minimumIntegerDigits: 2, 
        useGrouping: false
    });

    var articles_table = document.getElementById("articles_table");

    $.getJSON("/articles/articles.json", function(data){
        for (let i=start; i<Math.min(data.length, start+total); i++) {
            var e_tr = document.createElement('tr');
            
            var e_td1 = document.createElement('td');
            e_td1.textContent = "読み込み中...";

            var e_td2 = document.createElement('td');
            e_td2.textContent = data[i].year+"/"+formatter.format(data[i].month)+"/"+formatter.format(data[i].day);

            var e_td3 = document.createElement('td');
            e_td3.textContent = "詳しく見る";

            var e_a = document.createElement('a');
            e_a.href = data[i].link;
            e_a.appendChild(e_td3);

            e_tr.appendChild(e_td1);
            e_tr.appendChild(e_td2);
            e_tr.appendChild(e_a);

            articles_table.appendChild(e_tr);

            get_data_for_table(data[i], e_td1);
        }
    });
}

function get_data_for_table(data, e_td1) {
    const doc = fetch(data.link)
    .then((res) => res.text())
    .then((text) => {
        return new DOMParser().parseFromString(text, "text/html");
        })
    .then((doc) => {
        e_td1.innerHTML = doc.getElementById("article_title").innerText;
    }
);}
